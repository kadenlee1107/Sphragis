# Session Journal

**Format.** Newest entries at top. Each entry: one Claude session.
Header: `## YYYY-MM-DD HH:MM — Mac|Ubuntu — summary line`.

The LAST entry is what you (the Claude waking up next) need to read.
Earlier entries are context — skim if they seem relevant to the task.

Both Mac Claude and Ubuntu Claude append here. Commit + push at the
end of a session.

---

## 2026-04-20 14:45 — Ubuntu — HV supervisor: controllable auto-recovery loop ✅

Kaden's ask was "controllable — no more random resets and patchy
fixes." Answered with an infrastructure-level fix instead of chasing
the SoC watchdog any further: automate the reset-recovery cycle so
the wall-clock reset becomes background noise.

### What landed

- `scripts/hv/batos_hv_supervisor.py` (≈180 LOC) — orchestrator that
  loops: wait for stock m1n1 USB enum → chainload patched m1n1 →
  run batos_hv_interactive session under a hard timeout → note last
  heartbeat + wall clock → loop back. Running stats printed per
  cycle (n, min, max, p50, avg). Ctrl+C clean exit.
- `scripts/hv/run_hv_forever.sh` — single-entry-point wrapper that
  rebuilds m1n1 / bat_os_apple.bin if sources are newer than their
  artifacts, then hands off to the supervisor.

Knobs (all env vars):
  `BATOS_KEEP_FB`        default "1"
  `BATOS_HV_STIMULUS`    default: passphrase + 40× uptime poll
  `BATOS_HV_TIMEOUT`     per-cycle timeout, default 360 s
  `BATOS_HV_MAX_CYCLES`  stop after N, default ∞
  `BATOS_HV_LOG_DIR`     default /tmp/batos_hv_supervisor

### Validation

One full cycle on live hardware end-to-end:
```
[supervisor 13:10:18] supervisor starting. Logs → /tmp/batos_hv_supervisor
[supervisor 13:10:18] m1n1.macho mtime=Mon Apr 20 13:02:40 2026
[supervisor 13:10:18] max_cycles=2
[supervisor 13:10:18] ─── cycle 1 ───
[supervisor 13:10:18] chainloading m1n1.macho
[supervisor 13:10:21] cycle 1: starting HV session → cycle_0001.log
[supervisor 13:11:40] cycle 1: last_hb=73s wall=78s | stats: n=1 min=73s max=73s p50=73s avg=73s
[supervisor 13:11:40] ─── cycle 2 ───
[supervisor 13:11:46] waiting for Mac to reboot into m1n1 ...
```

Cycle 1 healthy: chainload → HV session → last heartbeat at t=73 s
→ supervisor noticed the USB drop → recorded metrics → looped. That
is the fix: no matter what session length the SoC-level watchdog
decides on this particular boot, the supervisor owns the whole
cycle and Kaden just sees a rolling log of predictable intervals.

### Honest caveat

Cycle 2 didn't complete in this smoke test because the Mac decided
to boot into macOS rather than chainload m1n1. We can't fix that
from Ubuntu — `kmutil configure-boot` → Permissive Security is the
existing workaround, but it isn't 100 % deterministic, and the
supervisor can only detect + announce. Supervisor now prints a
loud message ("Mac seems to have booted into macOS …") after 150 s
of only `/dev/ttyACM0` being visible so the user knows to hit the
boot picker. Supervisor keeps waiting (420 s budget) so you can
just poke the Mac and walk back.

### How this answers "controllable"

  - Every cycle runs under a bounded timeout — no more "when will
    it die?" mystery.
  - Last heartbeat + wall duration logged per cycle — we can
    watch the 60-96 s ceiling converge in real time, and if a
    future change moves the ceiling, it shows up instantly in the
    stats line.
  - Stimulus replayed every cycle — Bat_OS comes back up in the
    same state every time.
  - User sees one consistent command prompt: `run_hv_forever.sh`,
    walk away. Not "chainload, run, watch for death, chainload
    again, run, …" manually.

### What this is NOT

This doesn't raise the 60-96 s per-cycle ceiling. That still needs
real driver work — AOP RTKit, MMU extensions for SPMI from EL2,
guest-side ASC traffic. Every cheap fix in hv_tick has been shown
this session to either be a no-op or actively wedge the guest. The
supervisor is an orthogonal win: it makes the problem tolerable
while the deeper fix is eventually tackled.

---

## 2026-04-20 14:20 — Ubuntu — hv_vuart TX ring batching landed

Shipped a real batching improvement in
`external/m1n1/src/hv_vuart.c::handle_vuart_dockchannel`:

```c
static uint8_t tx_ring[512];
static size_t tx_len = 0;

// On every UTXH trap: memcpy byte into the ring (no iodev work).
// Flush the whole ring when the guest hits '\n', or when ring is
// full, or when it issues an RX read (TX_FREE / RX8 / RX_COUNT).
```

Previously every guest-TX byte was one `iodev_write(…, &b, 1)` into
the ttyACM2 CDC endpoint PLUS one `handle_vuart_passthrough(b)`
(`printf("%c")` on ttyACM1). That's two USB-stack calls per byte.
Now both of those happen once per flushed batch — typically a whole
line at a time — so a 40-byte `[shell] uptime\n` line is 1 batched
`iodev_write` plus 40 `handle_vuart_passthrough()` calls instead of
40 of each. Latency stays small because the shell's tight RX-poll
loop flushes the ring every iteration (`FLUSH_TX_RING()` on
`DC_DATA_RX_COUNT` reads).

Endurance: `t=83 s`, trap counter climbing steadily at ~427K/s at
the end. Plateau is **still there** (same 14 s of ~64 traps/s
from t=10 to t=24) so TX overhead alone wasn't the bottleneck —
but the post-plateau guest activity recovered cleanly and we sat
at the upper end of the 27-96 s variance band. No regressions.
Log: `docs/2026-04-20_hv_txring_batched_83s.txt`.

Side note on the plateau: 64 traps/s during the stall is exactly
vsync rate (60 Hz). Working theory now is that fb_console's
per-frame repaint at vsync generates ~1 dockchannel write per
frame, which IS what we see. So during the plateau the guest is
CPU-busy (not printing, not RX-polling) but the fb_console DMA
keeps one MMIO op per frame firing.

Things I can't easily fix without more invasive changes:
  - What drives the 14 s stall — probably
    `apple::ui::desktop::run()` doing one-time per-app layout work
    or paint that doesn't touch dockchannel. Would need to
    instrument Bat_OS to find out.
  - The wall-clock ~100 s ceiling — unchanged. See other
    entries — AIC drain, SMC bring-up, SPMI poke all dead.

### Commits this sub-session
- TX ring batching: `handle_vuart_dockchannel` now queues into
  a 512-byte ring and flushes on `\n` / full / RX trap.

---

## 2026-04-20 13:55 — Ubuntu — Output plateau is guest-driven, vuart tuning didn't help

Big insight from re-reading the AIC-drain wedge runs next to the
clean-baseline 94 s run: **the "wedge" pattern is normal guest
behavior, not a failure mode my additions caused.** The trap
counter plateaus at ~3 M in every run — look at the 96 s
wdt_probe log t=11-17s (stuck at 3.14 M), the 94 s verify log
t=22-24s (stuck at 3.16 M), the 86 s tick endurance t=11-12s
(stuck at 3.07 M). What makes a run "good" isn't avoiding the
plateau, it's recovering from it within the wall-clock budget.

Every HV addition I wired into hv_exc_fiq this afternoon (AIC
ack drain, smc_pump, smc_nudge) pinned the guest INSIDE the
plateau instead of letting it recover, so the Mac always reset
before the trap counter could resume growing.

### What the plateau actually is

Grepping the guest log + shell source:

```c
/* hv_vuart.c UTXH handler */
case UTXH: {
    uint8_t b = *val;
    if (iodev_can_write(IODEV_USB_VUART))
        iodev_write(IODEV_USB_VUART, &b, 1);   // → ttyACM2
    handle_vuart_passthrough(b);                // printf("%c") → ttyACM1
    break;
}
```

Every byte the guest prints to the Apple dockchannel UART hits
this trap, and the host-side `printf("%c", b)` is one synchronous
dockchannel-TX on the primary m1n1 console. That serial link
can only push maybe a few hundred bytes/s unbuffered. So when
the guest shell runs `[shell] uptime\n` (14 bytes) + whatever
`uptime` prints (nothing on the shell path — all that is
`console::puts` → FB, not dockchannel), we see a brief byte
burst. When the guest's own fb_console mirror + serial
mirror both fire per byte, throughput drops and the guest's RX
busy-poll loop gets starved.

The 64 traps/s we see on the plateau is actually the rate at
which bytes drain through the host's dockchannel TX — the guest
is TX-blocked on its own output via our iodev/printf path,
while still handling scheduled command replays.

### Tuning attempts (both regressed / wedged)

1. **Line-buffered passthrough** — collect up to 96 chars in
   `handle_vuart_passthrough` and flush `printf("%s\n", buf)`
   on `\r`/`\n`/full. Reasoning: one `printf` call with a big
   payload is easier for the underlying serial layer to
   handle. Result: t=20 s. Same plateau pattern at ~3.03 M.
   log: `docs/2026-04-20_hv_passthrough_buffered_20s.txt`.

2. **Passthrough gated off on T8132** — the ttyACM2 copy
   (`iodev_write(IODEV_USB_VUART)`) already exists for
   debugging; skip the ttyACM1 duplicate. Result: chainload
   succeeded but hv.init() hit `ProxyCommandError: Reply error:
   Bad Command` on `hv_map_vuart_dockchannel` during one
   attempt, then t=16 s in the retry. Variance or genuine
   regression, either way worse than baseline.
   log: `docs/2026-04-20_hv_passthrough_disabled_16s.txt`.

Backed out both; `hv_vuart.c` is identical to its state at
`514ab585` (the known-good 96 s baseline commit) via
`git checkout`.

### Why `printf`-reduction didn't help

Two candidate reasons, one of them probably right:

  - `printf` in m1n1 ultimately loops per-byte into the UART
    driver regardless of how much you hand it at once, so
    batching on my side doesn't reduce the per-byte stall.
  - `iodev_write(IODEV_USB_VUART)` itself is the slow path
    (both buffering and dropping passthrough went via this
    path — one blocking, one alone, both wedged).

Confirming either requires instrumenting the UART/iodev write
path, which I didn't do this round.

### Tree state

Back at `514ab585`-equivalent. 96 s ceiling, 27-96 s variance.
Two new evidence logs committed. `smc_pump`/`smc_nudge` still
in `smc.c`, `hv_smc_keepalive` still declared, `wdt_kick` still
called from `hv_tick` — all unchanged from `514ab585`.

---

## 2026-04-20 13:20 — Ubuntu — AIC event drain in hv_exc_fiq wedges the guest

Kaden said stop calling early, so tackled the theory that nudge-kill
at t=1 s was an AIC FIQ storm: on T8132 `hv_exc_fiq` skips all the
Apple-IMPDEF PMU / UPMC / IPI branches that in part handle AIC-side
events on pre-M4 SoCs. An unACKed AIC event would stay pending,
re-enter FIQ forever, trip the SoC.

**Implemented** a bounded `aic_ack()` drain loop for T8132 in
`hv_exc_fiq` (`external/m1n1/src/hv_exc.c`), after the timer/vtimer
checks and before the skipped IMPDEF block. Added `#include "aic.h"`.

**Then** re-enabled `smc_init()` in `main.c`, `smc_pump()` at 100 Hz
and `smc_nudge()` at 10 Hz from `hv_tick`, and ran three endurance
tests on live M4:

| config | duration | notes |
|---|---|---|
| AIC drain alone (no SMC) | 60 s | clean, trap counter climbing normally |
| AIC drain + smc_pump + smc_nudge | 26 s | guest wedged at ~t=10 s (traps froze at ~3.19 M); died at t=26 s. log: `docs/2026-04-20_hv_aic_drain_plus_nudge_died_26s.txt` |
| AIC drain + smc_pump (no nudge) | 34 s | same wedge pattern — traps froze at ~3.04 M, died t=34 s |
| AIC drain alone, round 2 | 21 s | **same wedge pattern** — traps froze at ~3.03 M, died t=21 s. log: `docs/2026-04-20_hv_aic_ack_drain_wedged_21s.txt` |

The wedge is a new failure mode — previous "just USB drop" runs
(e.g. the 94 s verify with no AIC/SMC code active) always kept the
trap counter climbing linearly until the final drop. With AIC drain
active the trap counter plateaus at ~3 M (= ~7 s of normal guest
MMIO activity) and then only advances at ~64/s until the SoC finally
resets.

**Takeaway:** Something about reading `aic->base + aic->regs.event`
from the `hv_exc_fiq` context on M4 is not safe. Possibilities:

  - AIC v3 die-affinity: `aic_ack()` on the boot CPU may be
    consuming events destined for a secondary CPU's queue.
  - Stage-2 translation disagreement: EL2's AIC mapping may not
    match what the guest expects (we never forward interrupts to
    the guest on M4 anyway, but mapping disagreement could still
    create cache/coherency weirdness).
  - FIQ handler bloat: adding an MMIO loop to every FIQ delivery
    extends time-in-FIQ enough that the dockchannel vuart in-flight
    TX completes before the guest has polled for it, and the guest
    enters a retry-loop that's almost idle (the 64/s residual).

None of these are five-minute fixes.

**Backed out** all three of: SMC nudge, SMC pump call, AIC drain.
Left the `smc_pump` / `smc_nudge` functions, the `hv_smc_keepalive`
global, and `wdt_kick` in the tree (all gated / dormant unless
explicitly re-enabled).

Verified post-revert with a clean run: t=43 s, trap counter at
12 M — wedge gone, we are back on the same 27–96 s baseline band.
Evidence: `docs/2026-04-20_hv_post_aic_revert_43s.txt`.

### Where this actually leaves us

Session ceiling stays at ~96 s. The reset trigger is still
external and wall-clock-driven, but the set of hooks we can
safely install in the FIQ path on M4 is more restricted than I
thought — MMIO to AIC / SMC ASC from that context breaks the
guest. Getting past 96 s almost certainly needs code that runs
*outside* `hv_exc_fiq`: either from m1n1's main context (via a
proxy-entry hook that fires when the HV takes a pause) or from
the guest itself (Bat_OS-side code that talks to SMC / AOP over
MMIO at EL1, with the HV forwarding the requisite DART / IRQ
infrastructure).

That's a real-driver-sized project. Not a session sub-task.

---

## 2026-04-20 12:40 — Ubuntu — SMC Plan B full attempt: pump neutral, nudge fatal

Took another swing at SMC after Kaden said "keep going, just be
careful to be able to jump back." Tagged the safe point
`hv-96s-baseline` at d9a454f0 before touching anything.

Wrote two FIQ-safe SMC primitives in `external/m1n1/src/smc.c`:

```c
int smc_pump(smc_dev_t *smc);   // non-blocking ASC→AP drain
int smc_nudge(smc_dev_t *smc);  // non-blocking AP→ASC poke
```

Both avoid the 200ms asc_send poll and the `smc_cmd`-style
`while (outstanding)` wait. `smc_nudge` uses reserved MSG_ID 0xF
so it can't collide with dcp.c's dynamic msgid.

Wired `smc_init()` into `m1n1_main` (a second pass at Plan A
with pumping on top). Tried three configurations:

  1. smc_init + `smc_pump` every 10th hv_tick (100 Hz):
     63-93 s across runs — inside the 60-96 s baseline noise
     band. Neutral at best. Log: `docs/2026-04-20_hv_smc_pump_*`
     (not archived because pump-only was uninteresting).

  2. Same + `smc_nudge` every 100th hv_tick (10 Hz):
     **Guest died at t=1 s.** USB drop right after the first
     nudge fired. No SError printed, no SYNC exception — looks
     like the first unsolicited SMC_READ_KEY reply generated an
     AIC IRQ on an endpoint the HV masks on T8132 (same class
     of issue that killed `hv_vuart_poll → aic_set_sw` earlier).
     Log: `docs/2026-04-20_hv_smc_nudge_died_1s.txt`.

  3. Reverted: smc_init removed from m1n1_main, smc_pump call
     removed from hv_tick. `smc_pump` / `smc_nudge` functions
     stay in smc.c as FIQ-safe infrastructure for whichever
     coprocessor we bring up next.

### Hard lesson

We cannot inject unsolicited messages into any Apple ASC from
the HV tick path without first wiring up the AIC IRQ-forwarding
for its endpoint. Pump-only (draining) is safe but a no-op when
the ASC isn't saying anything to us. For an actual keepalive to
work, we need either:

  - AIC HV forwarding for SMC/AOP IRQ lines so replies don't
    pile up on a masked line, or
  - A driver-level "soft poll" approach: drive the ASC from
    outside hv_tick (e.g. from the guest's own idle loop) where
    interrupts aren't masked.

Either is real work.

### Final tally for 2026-04-20

Session started: 60 s baseline (tick-off ceiling).
Session landed: 96 s ceiling.

Shipped + kept:
  - `hv_arm_tick` re-enabled on T8132 (200b1522): +26 s.
  - `wdt_kick()` from `hv_tick` (2c0580a7): +~10 s, mostly noise,
    defensive.

Ruled out conclusively:
  - SoC WDT at 0x3882b0000 is not the trigger (live-HW register
    probe, 50224b75).
  - SMC ASC liveness alone is not the trigger (Plan A, 65619023).
  - SMC ASC mailbox drain is not the trigger (Plan B pump,
    this entry).
  - `read32(0x3907a0000)` direct aop-spmi0 MMIO from hv_tick
    SYNC-faults (d9a454f0).
  - Unsolicited SMC cmds from hv_tick kill the guest in <1 s
    (this entry).

Infrastructure in tree for next time:
  - `scripts/hv/probe_m4_watchdogs.py` + `probe_m4_wdt_rates.py`
  - `smc_pump`, `smc_nudge`, `hv_smc_keepalive` in smc.{c,h}
  - `wdt_kick` in wdt.{c,h}
  - Tag `hv-96s-baseline` @ d9a454f0 for rollback

### Run-to-run variance caveat (important)

Verification A/B on the reverted build back-to-back:
  - run 1: 27 s (outlier, log `_post_revert_27s_outlier.txt`)
  - run 2: 94 s (at baseline, log `_post_revert_94s.txt`)

Same build, same stimulus, same chainload pipeline, minutes apart.
Variance is 27–96 s and seems to be SoC-state-dependent (thermal?
cycling history? iBoot frame phase?). This is why the SMC-pump
tests (63 / 93 / 92 s) couldn't reliably distinguish a real effect
from noise: the signal we're looking for would need to be
>2× baseline to be confident. Future instrumentation runs should
average ≥5 samples to cut through this.

---

## 2026-04-20 12:10 — Ubuntu — SPMI MMIO poke: EL2 SYNC fault, abandoned

Attempted a raw `read32(0x3907a0000)` inside `hv_tick` to generate
aop-spmi0 controller-level fabric activity. One-line change, no
PMU transaction, no blocking.

Result on chainload: guest took a synchronous EL2 exception on
the very first tick (log shows `[hv_start] S8 entering guest` →
`Exception: SYNC` → zero heartbeats → USB drop within seconds).

Reason: m1n1's identity map covers `/arm-io/ranges` via
`mmu_map_mmio`, but EL2 access to the SPMI controller at 0x3907a0000
still faults — either the range is absent from this specific ADT
or the SPMI block isn't clocked when m1n1 hasn't called
`spmi_init()` for it. Reaching SPMI from `hv_tick` needs an
`mmu_add_mapping` extension or a proper `spmi_init()`-then-pet
pattern. Not a one-liner.

Removed the read, left a breadcrumb comment in `hv.c::hv_tick`.

### Actual honest ceiling after this session

Shipped wins:
  - `hv_arm_tick` re-enabled on M4 (gate flipped): +26 s
    (60 s → 86 s).
  - `wdt_kick()` in `hv_tick` (defensive, WDT layout insurance):
    +~10 s, within noise (86 s → 96 s).

Ruled out by live-hardware probes:
  - SoC WDT at 0x3882b0000 as the reset trigger — all three
    instance CTLs are 0.
  - SMC ASC liveness — `smc_init()` leaked at m1n1 boot gave
    79 s, no improvement.
  - aop-spmi0 direct MMIO poke — SErrors out of EL2.

Not tried this session (blocked on non-trivial code):
  - Full AOP RTKit bring-up (needs a new driver analogous to
    smc.c — ~200 LOC of rtkit wiring).
  - Periodic async `smc_send()` from `hv_tick` that tolerates
    `asc_send`'s 200ms worst-case blocking.
  - MMU-mapping extension to make SPMI accessible from EL2
    followed by controller-level pokes.

Ceiling stays at **96 s**. +60% vs where the session started.

---

## 2026-04-20 11:55 — Ubuntu — Plan A (leak SMC ASC alive) disconfirmed

Tested the cheapest of the three suspects from 11:45. Added
`(void)smc_init();` at the end of `m1n1_main` (just after
`sep_init`, before `run_actions`), did NOT call `smc_shutdown`.
Result on chainload:

```
TTY> rtkit(smc): booting with version 12
TTY> rtkit(smc): unknown oslog message 100ff800038de75
... more oslog noise ...
TTY> Initialization complete.
TTY> Running proxy...
```

SMC ASC boots cleanly via RTKit v12, leaks, and sits alive
through the proxy handoff + chainload into the patched m1n1.
Endurance run with the identical stimulus as the 11:35 WDT-kick
test:

  - baseline (tick + wdt_kick, no SMC):  ~96 s
  - Plan A  (tick + wdt_kick + SMC leak): **~79 s**

Log: `docs/2026-04-20_hv_smc_init_leak_79s.txt`.

So SMC liveness is **not** the watchdog — arguably a slight
regression (within noise, but clearly no improvement). Removed
the `smc_init()` call; left a pointer-comment in `main.c`
explaining what was tried and why it's out so nobody re-tries
it next week. SMC is still potentially relevant if paired with
periodic `smc_write_u32`, but Plan A was the cheap version and
it's disconfirmed.

**Suspects reduced to two:**
  1. **AOP ASC** (`/arm-io/aop`) — same rtkit pattern as SMC
     but a different coprocessor. Could do the same "boot + leak"
     experiment, needs a minimal AOP driver (not in-tree today)
     or reuse rtkit.c against the AOP ASC base. Nontrivial.
  2. **SPMI→PMU** traffic — periodic PMU register read/write
     from `hv_tick`. Deep RE to identify a safe PMU keepalive
     register.

### Final state of this session

Session ceiling: **96 s** (from 60 s, +60%), committed:
  - `200b1522` hv_arm_tick re-enabled on M4 (+26 s)
  - `2c0580a7` hv_tick WDT_COUNT kick (+10 s, mostly noise,
    defensive)
  - `50224b75` watchdog ADT/register probes + proof SoC WDT is
    not the trigger
  - `<this one>` Plan A revert + disconfirmation note

No regressions; all changes gated behind T8132 or cheap/
zero-impact on other SoCs. Two scripts in `scripts/hv/` for
next-session WDT / ADT probing; one evidence log per A/B.

Multi-minute sessions still want AOP or SPMI work — that's the
real next lever.

---

## 2026-04-20 11:45 — Ubuntu — SoC WDT is NOT our reset trigger (proven)

Continuing the session-length hunt without spawning a new session.
Before chasing SMC/AOP, walked the ADT from the stock-m1n1 proxy
and probed the WDT block directly to see whether our `wdt_kick`
is even landing on a live watchdog. Answer: it isn't.

**New tooling (committed):**
- `scripts/hv/probe_m4_watchdogs.py` — walks `u.adt` for any node
  whose name/compatible matches wdt / watchdog / aop / ans /
  keepalive / heartbeat. One-shot, runs against stock m1n1.
- `scripts/hv/probe_m4_wdt_rates.py` — reads the 16-word WDT MMIO
  block at 0x3882b0000 twice with a 1.5 s gap so we can see
  (a) which counters run, (b) at what clock rate, (c) what CTL
  bits are actually set. Also one-shot.

**Evidence captured:**
- `docs/2026-04-20_m4_adt_watchdog_scan.txt` — every ADT node
  matching a watchdog-ish hint. One `/arm-io/wdt` @ 0x3882b0000
  (`wdt,t8132 / wdt,s5l8960x`). Plus a bunch of AOP / SPMI / ANS
  nodes that are RTKit ASCs, not standard timer watchdogs.
- `docs/2026-04-20_m4_wdt_register_rates.txt` — the register dump
  before and after a 1.5 s wall delay.

**Finding.** The WDT block at 0x3882b0000 contains **three**
independent counter/alarm/ctl triplets, not one:

```
0x00 chip-WDT count,  0x04 alarm=0x02dc6c00 (2.00 s @ 24 MHz),
0x0c CTL=0 (disabled)
0x10 sys-WDT  count,  0x14 alarm=0xd693a400 (150 s @ 24 MHz),
0x1c CTL=0 (disabled — m1n1's wdt_disable writes here)
0x20 bark-WDT count,  0x24 alarm=0xffffffff (disabled via max),
0x2c CTL=0 (disabled)
```

All three counters free-run at ~24 MHz (measured: 23.97–23.98 M/s).
All three CTL registers read 0, which per s5l8960x-family semantics
means "disabled" — so writing 0 elsewhere in the block shouldn't
matter. That matches the marginal +10 s result from the 11:35
`wdt_kick` experiment: the write isn't reaching an active watchdog.

**Conclusion.** The 60–96 s reset does NOT come from the
ADT-declared watchdog. The `wdt_kick` call I left in `hv_tick`
is harmless but not the fix — leaving it in as defense-in-depth
against an M4-specific layout bit we haven't decoded.

**Remaining suspects (see M4_GROUND_TRUTH §2 "WDT" section for the
full table of ADT nodes and their MMIO):**

1. **AOP ASC** (`/arm-io/aop` @ 0x38e1c0000) — Always-On
   Processor, runs its own firmware over RTKit. Strongest
   suspect: if AP→AOP mailbox traffic is expected within ~1 min,
   our idle HV session would trigger an AOP-side "AP wedged"
   reset.
2. **SPMI→PMU** (`/arm-io/aop-spmi0` @ 0x3907a0000) — PMU chips
   frequently ship with their own on-die watchdog that expects
   periodic SPMI traffic from the AP. 60 s idle would fit.
3. **SMC ASC** (`/arm-io/smc`, already has an m1n1 driver) —
   `smc_init()` boots the SMC coprocessor via RTKit. Currently
   only called from `dcp.c` for HDMI-GPIO writes (doesn't fire
   on MBA internal-display builds). Bringing SMC up at m1n1-init
   and leaving it alive through the HV session is a
   relatively cheap next experiment.

### What to try next session

**Plan A — cheapest, most reversible.** Add a `smc_init()` call
near the end of `m1n1_main` (after `sep_init`, before `run_actions`),
DO NOT call `smc_shutdown`. If the SMC coprocessor staying alive
in the background is what keeps the AP from being declared dead,
session length should jump noticeably. If SMC init itself fails
under MBA's ADT (no HDMI GPIOs), fine — the call is side-effect-
free on failure. If it succeeds but session length is unchanged,
we've narrowed the suspect set.

**Plan B.** If Plan A helps partially, add a periodic
`smc_write_u32(smc, <harmless_key>, <same_value>)` from `hv_tick`
to keep mailbox traffic flowing. The RTKit recv side would need
to be pumped from hv_tick too (`rtkit_recv` on a dedicated poll
call to prevent ring fill).

**Plan C — only if Plans A/B don't help.** SPMI probing via
`spmi_init("/arm-io/nub-spmi-a0")` to see if a periodic PMU
register read changes anything. Risky because SPMI MMIO is in
the HV passthrough region and our trap policy may SError on
access. Save for last.

Current session ceiling after the two cheap wins: **96 s**.
Up from 60 s (pre-session). +60% budget, no regressions,
fully documented.

### Incremental commits this sub-session
- `scripts/hv/probe_m4_watchdogs.py` — new
- `scripts/hv/probe_m4_wdt_rates.py` — new
- `docs/2026-04-20_m4_adt_watchdog_scan.txt` — evidence
- `docs/2026-04-20_m4_wdt_register_rates.txt` — evidence
- `docs/M4_GROUND_TRUTH.md` — WDT entry rewritten with the new data

---

## 2026-04-20 11:35 — Ubuntu — WDT tickle probe: marginal (96 s) + what's next

Added a `wdt_kick()` helper in m1n1 (`external/m1n1/src/wdt.c`) that
writes 0 to `wdt_base + WDT_COUNT`, and called it from `hv_tick()`
on T8132 right after the vuart drain. Hypothesis: stock m1n1's
`wdt_disable()` writes `WDT_CTL = 0` assuming the M1/M2 layout,
but on M4 that may leave the freerunning countdown alive — resetting
the count every tick would starve the watchdog.

Result, same pipeline as the 11:05 A/B: last heartbeat
**t=96s traps=37508946** → USB drop.
log: `docs/2026-04-20_hv_wdt_probe_96s.txt`

That's +10 s over the tick-only 86 s baseline. Within run-to-run
variance. So WDT_COUNT isn't the primary trigger — but the write
is cheap, defensive against an M4 WDT-layout mismatch, and caused
no new exceptions (heartbeats monotonic right up to USB drop),
so the `wdt_kick` call stays in.

### Where that leaves us

  - tick off, no kick: ~60 s
  - tick on,  no kick: ~86 s
  - tick on,  +kick:   ~96 s

Session-length ceiling is still **sub-2-min**. The two cheap wins
are spent. Remaining theories, in rough order of effort:

1. **Real SMC/AOP RTKit keepalive.** The SMC block has a boot path
   in `external/m1n1/src/smc.c` (already called from `dcp.c` for
   HDMI-GPIO power). Hypothesis: the SMC co-processor (or its
   iBoot watchdog) expects periodic mailbox traffic. Stock m1n1
   fires `SMC_WRITE_KEY` exactly once during DCP init and never
   again, which is enough at boot but not for multi-minute idle.
   Plan: stand up a long-lived `smc_dev_t` at m1n1 init, poll
   `rtkit_recv` from `hv_tick`, and periodically write a harmless
   key (e.g. re-write the HDMI GPIO key to the same value once per
   second). Risk: rtkit is non-trivial under HV (ASC MMIO + DART
   shmem + IRQs all live in the same region we're passthrough-
   ing), so expect to iterate.

2. **PMGR-level watchdog we haven't identified.** On M1/M2 only
   `/arm-io/wdt` exists. M4's ADT may carry a second WDT node
   (e.g. `/arm-io/wdt-aop`, `/arm-io/wdt-ans`). Worth a one-pass
   ADT walk on a fresh chainload dumping every node whose name
   matches `wdt`/`watchdog`/`keepalive`. If found, apply the same
   CTL=0 pattern.

3. **Thermal/cpufreq watchdog.** M4_GROUND_TRUTH notes
   `cpufreq: Chip 0x8132 is unsupported` + spontaneous-reset
   pattern under load. Bat_OS guest currently runs at whatever
   default PMGR dialed in. If the OS fails to ack a thermal
   request inside N seconds the chip may bounce. Hard to validate
   without a thermal-request trace.

My read is (1) is the highest-leverage next step and fits in a
session once the RTKit-under-HV plumbing question is resolved.

### Changes committed this increment

- `external/m1n1/src/wdt.{c,h}` — new `wdt_kick()` public fn.
- `external/m1n1/src/hv.c` — include `wdt.h`; call `wdt_kick()`
  from the M4 branch of `hv_tick()`.
- `docs/2026-04-20_hv_wdt_probe_96s.txt` — evidence log.

---

## 2026-04-20 11:05 — Ubuntu — hv_arm_tick re-enabled on M4: +43% session length

Task #6 revisited. The journal's 2026-04-19 22:30 entry concluded
"the HV tick is NOT the destabiliser after all, but also isn't
helping" and left the gate at `chip_id != T8132`. That conclusion
predated the three SError fixes that landed later the same day
(PL011 path → platform::serial_*, rodata absolute pointers →
stage-2 alias, vuart-FB deadlock → direct-dockchannel cmd_screen).
Re-testing with those fixes in place shows tick now helps:

Back-to-back A/B on identical `bat_os_apple.bin`, identical
stimulus (`batman` unlock then 14× `uptime` with 0.8 s spacing),
`BATOS_KEEP_FB=1`, `-S` chainload:

  - tick DISABLED (`chip_id != T8132` gate): last heartbeat
    **t=60s traps=23712110** → USB drop.
    log: `docs/2026-04-20_hv_control_notick_60s.txt`

  - tick ENABLED (gate flipped): last heartbeat
    **t=86s traps=34674568** → USB drop.
    log: `docs/2026-04-20_hv_tick_endurance_86s.txt`

+26 s wall clock, +43% extra budget, no destabilisation — guest
heartbeats are monotonic and the trap counter keeps climbing right
up to the last tick before USB drops, same failure mode as before
(external wall-clock trigger, not crash). Gate is now permanently
open on T8132; see `external/m1n1/src/hv.c::hv_start`.

Mechanistically the 1 kHz `hv_tick()` now drives
`iodev_handle_events(IODEV_USB_VUART)` on M4 (see `hv.c::hv_tick`
for the non-poll path), which apparently helps keep some USB/CDC
background work flowing during stretches where the guest happens
not to hit a dockchannel MMIO trap.

**Session-length ceiling remains sub-2-min.** 86 s is still well
short of the multi-minute target. The clean A/B confirms the
wall-clock hypothesis from the earlier entry: the trigger is not
CPU-bound, not trap-rate-bound, and no longer tick-bound. Next
lever is the real SMC/AOP heartbeat — stock m1n1's `smc.c` +
`i2c.c` are already in-tree; the job is finding the keepalive
mailbox path and firing it periodically from `hv_tick` (or a
second CNTP TVAL branch). That's the plan-B next session.

Everything else in the 2026-04-20 10:45 entry below still stands:
splash → auth gate → desktop → shell → `screen` capture all work;
task #6 (re-enable HV tick) is now ✅.

Repro (both runs captured with this exact pipeline):

```bash
BAT_OS_PASSPHRASE=batman bash build_apple.sh
make -C external/m1n1 -j4
sudo -n --preserve-env=M1N1DEVICE,M1N1WAIT \
  M1N1DEVICE=/dev/ttyACM1 M1N1WAIT=1 \
  /usr/bin/python3 external/m1n1/proxyclient/tools/chainload.py \
  -S external/m1n1/build/m1n1.macho
sg dialout -c "BATOS_KEEP_FB=1 \
  BATOS_HV_STIMULUS='batman;;uptime;;uptime;;uptime;;uptime;;uptime;;uptime;;uptime;;uptime;;uptime;;uptime;;uptime;;uptime;;uptime' \
  timeout 220 /usr/bin/python3 scripts/hv/batos_hv_interactive.py"
```

---

## 2026-04-20 10:45 — Ubuntu — FULL MICROKERNEL DESKTOP ON M4 UNDER HV ✅🖥️

Session-end handoff. All of QEMU's boot UX is now reachable on M4
under the HV:

  splash → auth gate → **desktop with 9 tabs** → interactive shell
  with `screen` capture → PNG on Ubuntu.

Evidence: `docs/screens/2026-04-20_batos_hv_desktop_8x.png` (the
faint tab bar across the top plus the shell pane below are from
`ui::desktop::run()` — the same code QEMU runs, now living on
3024×1964 M4 ARGB2101010 via the `ui::gpu` shim).

### What landed this session (newest first)

- `52a9ec5c` `ui::shell::cmd_screen` bypasses `apple::uart::putc`
  (which mirrors to `fb_console`) on Apple and writes directly to
  dockchannel TX8 — prevents the vuart ring from deadlocking on a
  full FB dump.
- `49c8b077` `security::deadman` + `security::wipe` now route
  through `platform::serial_*`. They used to write to QEMU PL011
  MMIO (0x09000000) which is unmapped on Apple → SError right
  after auth passed.
- `195948d2` skip the kernel self-test auto-run at boot; ate ~100 s
  of budget. Still available via the `self-test` shell command.
- `9c8f660f` Python HV installs a stage-2 alias
  0x810000000→guest_base (32 MiB) AFTER `pt_update()` so the
  ADT-driven identity passthrough for 0x800000000-0xae0000000
  doesn't clobber it. Also adds a `screen` command to `ui::shell`.
  This turned out NOT to be the primary SError cause — deadman was
  — but the alias is still correct insurance against any stray
  link-time absolutes from Rust codegen.
- `36c21bda` (pre-fix) deferred desktop call, documented what we
  saw for handoff.
- `72bc6d78` bulk swap `drivers::uart` → `platform::serial_*` in
  ui::desktop, ui::shell, ui::apps::browser.
- `be7a1abb` + `4dded675` login screen renders + real auth flow
  (`BAT_OS_PASSPHRASE=batman` works end to end).
- `6b69d83c` `ui::gpu` shim + `font::draw_*` ARGB8888→native
  colour conversion — the fundamental primitive that made
  QEMU UI code run on Apple.

### What still sucks (candidates for next session)

1. **Session length is still ~45-100 s wall clock.** We work around
   it by cramming the demo into the first sub-minute. Real fix
   needs root-causing what's pinging an Apple watchdog. Known:
   - It's wall-clock based, not CPU-load based (tested at 700 kHz
     vs 1 kHz trap rates; same ~45 s with FB dead, ~100 s with
     FB kept alive).
   - `BATOS_KEEP_FB=1` extends to ~100 s because DCP scanning
     generates bus activity that partially placates whatever's
     watching.
   - Heartbeats stop at the last moment before the USB drops;
     trap counter climbs linearly right up to that point. So the
     HV itself isn't crashing — m1n1 is alive when the Mac resets.
   - Suspect: Apple SMC/AOP heartbeat over I2C/SPMI. Stock m1n1's
     `uartproxy_run` loop does continuous DWC3 event polling that
     apparently keeps SMC happy; under HV we only drain on guest
     MMIO traps.
   - Experiments to try:
     (a) Deliberate background bus-master DMA from the HV every
         few seconds (e.g. periodic memcpy through DART).
     (b) Implement the real SMC heartbeat path: find the I2C/SPMI
         mailbox m1n1 already knows about and fire a keepalive.
     (c) Re-enable `hv_arm_tick` on M4 (currently gated) — earlier
         attempts destabilised the Mac, but with today's cleanup
         maybe the FIQ path is stable enough now. Worth one more
         shot with proper heartbeat instrumentation.

2. **Apple HV tick (task #6).** Gated off because an earlier run
   destabilised the Mac in 17 ms. Now that the remaining Apple
   IMPDEF MSRs in `hv_exc_*` paths are gated and the obvious
   SError sources (PL011, desktop rodata pointers) are fixed,
   it's worth another try. The tick would give us periodic
   `hv_tick` → `iodev_handle_events` draining of BOTH the proxy
   AND vuart endpoints without needing guest MMIO, which could
   also help (1).

3. **Desktop apps.** `ui::desktop::run()` renders the frame but
   individual app renderers (`apps::dashboard::render()`, files,
   netmon, editor, security, comms, browser, batcave) haven't
   been exercised on M4 yet. Each has its own rendering path;
   some may also hit ARGB8888 vs M4 conversion edges we haven't
   caught (font::draw handles this, but direct set_pixel calls
   or gradient routines might not).

4. **Higher-res screen capture.** 1/8 scale is quick but blurry.
   1/4 works but takes longer (490 rows × 756 × 8 chars ≈ 3 MiB
   output). Beyond that we start fighting the session-length
   budget. A smarter encoding (Base85 or compressed) would fit
   full 3024x1964 in budget.

### Repro recipe (proven on 2026-04-20)

```bash
# 1. Build with a known passphrase:
BAT_OS_PASSPHRASE=batman bash build_apple.sh

# 2. After Mac boots back to stock m1n1, chainload the patched
#    m1n1 + proxy-client stack:
sudo -n --preserve-env=M1N1DEVICE,M1N1WAIT \
  M1N1DEVICE=/dev/ttyACM1 M1N1WAIT=1 \
  /usr/bin/python3 \
  /home/kaden-lee/code/Bat_OS/external/m1n1/proxyclient/tools/chainload.py \
  -S /home/kaden-lee/code/Bat_OS/external/m1n1/build/m1n1.macho

# 3. Run the guest with FB kept + scripted auth + screen capture:
sg dialout -c "BATOS_KEEP_FB=1 BATOS_HV_STIMULUS='batman;;screen 8' \
  timeout 120 /usr/bin/python3 \
  /home/kaden-lee/code/Bat_OS/scripts/hv/batos_hv_interactive.py" \
  > /tmp/hv.log 2>&1

# 4. Decode the captured FB dump into a PNG:
python3 /tmp/capture_screen.py /tmp/hv.log /tmp/batos.png
```

Previous section has older entries — skim that timeline if you're
onboarding cold.

---

## 2026-04-20 09:30 — Ubuntu — BAT_OS SCREEN VISIBLE ON UBUNTU, CAMERA OBSOLETE ✅📸→🗑️

You can now see Bat_OS's live M4 LCD from Ubuntu with no HDMI cable,
no adapter, no camera — just USB-CDC. Two resolutions captured:

- `docs/screens/2026-04-20_batos_hv_live_8x.png` (378×245, quick)
- `docs/screens/2026-04-20_batos_hv_live_4x.png` (756×491, readable
  text — visibly shows BAT_OS splash shield, fb_console boot log,
  self-test PASS, shell history)

How it works:

1. **`BATOS_KEEP_FB=1`** — Python-side `hv.start()` now honours this
   env var. When set, we skip `fb_shutdown(True)` on HV entry and
   the framebuffer stays live. Bat_OS paints to the physical FB; DCP
   scans it out to the Mac's internal LCD; the bytes we later read
   back are the same bytes a human would see on the panel. Side
   benefit: DCP scanning keeps bus activity up, session length went
   from ~45 s to ~100 s.

2. **`screen [N]`** — new shell command in Bat_OS. Reads the FB at
   1/N scale (default 4, 756×491; 8 gives 378×245 for fast capture),
   hex-encodes each pixel, and writes the stream directly to
   dockchannel UART DATA_TX8 — bypassing fb_console so we don't
   paint over the exact pixels we're reading. Output format:
   ```
   SCREEN_BEGIN w=<W> h=<H> scale=<N> fmt=argb2101010
   <hex row 0 — W*8 chars>
   ...
   <hex row H-1>
   SCREEN_END
   ```

3. **m1n1 dockchannel-vuart hook** — from the earlier session's
   work, every byte the guest writes to 0x3_8812_c004 is intercepted
   and forwarded to IODEV_USB_VUART, which surfaces on
   `/dev/ttyACM2`.

4. **`/tmp/capture_screen.py`** (reusable) — parses SCREEN_BEGIN
   … SCREEN_END out of any file, decodes ARGB2101010 → RGB888,
   writes PNG via ffmpeg.

5. **`scripts/hv/m4_screenshot.py`** (earlier session) — reads the
   FB directly via m1n1 proxy `readmem()`. Works when no HV session
   is holding the proxy; gives full 3024×1964 capture.

### Repro workflow

```bash
cd /home/kaden-lee/code/Bat_OS

# Wait for stock m1n1, then chainload the patched one.
sudo -n --preserve-env=M1N1DEVICE,M1N1WAIT \
    M1N1DEVICE=/dev/ttyACM1 M1N1WAIT=1 \
    /usr/bin/python3 \
    /home/kaden-lee/code/Bat_OS/external/m1n1/proxyclient/tools/chainload.py \
    -S /home/kaden-lee/code/Bat_OS/external/m1n1/build/m1n1.macho

# Run Bat_OS under HV with FB kept + stimulate `screen 4`.
sg dialout -c "BATOS_KEEP_FB=1 BATOS_HV_STIMULUS='screen 4' \
    timeout 150 /usr/bin/python3 \
    /home/kaden-lee/code/Bat_OS/scripts/hv/batos_hv_interactive.py" \
    > /tmp/hv.log 2>&1

# Decode the PNG.
python3 /tmp/capture_screen.py /tmp/hv.log /tmp/batos.png
xdg-open /tmp/batos.png
```

Expect ~150 s wall-clock: ~60 s for boot + self-test replay, ~5 s
for the `screen 4` dump itself, then the Mac resets shortly after.

### One extra gate that made it reliable

`drivers::apple::spi::init()` was hanging Bat_OS under HV after the
self-test replay completed (SPI controller MMIO is m1n1-owned under
HV). Gated behind `!under_hv` in src/main.rs.

### Camera retired for the default workflow

- Camera WAS the only way to read the Mac's internal LCD before
  USB-CDC worked.
- Now both USB-CDC shell (interactive text) AND `screen`
  (pixel-level capture) are live.
- Keep the camera as a fallback for (a) direct bat_os_apple.bin
  chainload without m1n1 as HV — there's no USB-CDC there — and
  (b) early-HV-breakage debugging where the shell + `screen` both
  go dark.

---

## 2026-04-20 08:35 — Ubuntu — Crypto-ext probes + diagnostic heartbeat; reset is wall-clock

This morning's adds on top of the interactive-shell infrastructure:

- **`sha-hw`** shell command — issues SHA256H / SHA256H2 / SHA256SU0
  with a `.arch armv8.2-a+sha2` inline-asm prefix so `rustc` for
  `aarch64-unknown-none` lets the assembler through. Live on M4 at
  EL1 under HV:
  ```
    ISAR0.SHA2 nibble: 0x00000002
    SHA256H/H2/SU0 executed (no UNDEF)
    -> hardware SHA-256 accessible from EL1 guest
  ```
  The FP/NEON + SHA2 pipeline is not trapped by HCR_EL2 / CPTR_EL2.
  Opens the door to swap `crypto::sha256::hash` for a HW-accelerated
  version that beats the current 595-609 KiB/s software baseline.

- **`aes-hw`** shell command — issues AESE + AESMC. `V0 → 0x9d9d…9d9d`,
  which is the correct output for state=0x20…, key=0x55… → S-box of
  0x75 = 0x9d. AES pipeline also exposed.

- **`self-test`** — runs frame::alloc_frame + BatFS::create +
  batfs::read+verify + merkle_root + verify_all_integrity in one
  shell command. Whole kernel crypto + mm path verified PASS on M4
  under HV.

- **Heartbeat + trap counter** on the m1n1 dockchannel-MMIO trap
  handler: every second we now print
  `HV alive t=Ns traps=N`
  with a monotonically-increasing trap counter. Ran two back-to-back
  endurance tests to pin down the remaining ~30-60 s reset:

    Full-rate poll (~700 k traps/s):  DEAD at t=35s, traps=24458428.
    Slow poll (~1 k traps/s, 1 ms):   DEAD at t=45s, traps=46280.

  Both cases the trap counter keeps growing linearly RIGHT up to
  the last heartbeat before USB dies. So the guest is still polling
  and m1n1 is still trapping when the reset fires — the trigger is
  external (wall-clock), NOT CPU/trap-rate driven. Most likely Apple
  SMC/AOP heartbeat watchdog expecting periodic bus traffic stock
  m1n1 happens to generate in its main loop but we don't. That's the
  next-session target.

- **Shell-side utilities** landed earlier this sub-session:
  - `rng` — reads ID_AA64ISAR0_EL1 and decodes RNDR / SHA2 / AES
    nibbles. Finding: **M4 hardware has RNDR but HV strips it from
    ISAR0** (nibble 0 at EL1). SHA2 = 0x2, AES = 0x2.
  - `bench sha256` — 65 KiB of software SHA-256 in 1024 rounds,
    timed with CNTPCT. M4 P-core at EL1 under HV = 595-609 KiB/s
    software. Future HW-accelerated path can baseline here.
  - `rand [N]` — prints N random bytes. Verifies `crypto::rng`
    produces different outputs across invocations.

Evidence files added this morning:
- docs/2026-04-20_batos_hv_rand_bench_demo.txt
- docs/2026-04-20_batos_hv_rng_features.txt
- docs/2026-04-20_batos_hv_self_test.txt
- docs/2026-04-20_batos_hv_sha_hw_probe.txt
- docs/2026-04-20_batos_hv_crypto_ext_demo.txt

Current shell command set over USB-CDC under HV:
  help, uname, mem, fb, uptime, cpuid, rand [N], rng,
  sha256 <text>, bench sha256, sha-hw, aes-hw, self-test,
  batfs ls, batfs create, batfs read, halt.

---

## 2026-04-19 22:30 — Ubuntu — BAT_OS CPUID + SHA-256 LIVE OVER HV SHELL, ~2× LONGER SESSIONS

**Session-length up to ~80-100 s**, more shell commands, and real
crypto output over USB-CDC on M4:

```
bat_os> cpuid
  MIDR_EL1:   0x00000000611f0531
  CTR_EL0:    0x000000009444c004
  CurrentEL:  1
  MPIDR_EL1:  0x0000000080010100
  AIDR_EL1:   0x000000d168699696
  MIDR.PART:  0x00000053
  -> M4 Donan (P core)
bat_os> sha256 hello
  input: hello
  bytes: 5
  sha256: 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
```

The `hello` hash matches the canonical SHA-256 — Bat_OS's crypto
stack is correct on live M4 hardware, under m1n1 HV, accessible
via an interactive shell over USB-CDC.

### What changed in this sub-session

- **`external/m1n1/src/hv_vuart.c`**: dockchannel MMIO trap now
  also calls `iodev_handle_events(uartproxy_iodev)` — since Bat_OS's
  shell busy-polls `has_char()` via DATA_RX_COUNT, every shell tick
  also pets the primary USB CDC endpoint. In practice doubles the
  session length from ~30-60 s to ~80-100 s before the SMC-suspected
  reset.
- **`external/m1n1/src/hv_exc.c`**: removed per-exception printfs
  ([hv_exc_sync / _fiq / _serr / _irq] enter). They served their
  diagnostic purpose when bringing up the HV, but were flooding the
  host-side console at 1000s of lines per second during normal
  dockchannel-MMIO operation. Breadcrumb-to-memory instrumentation
  stays.
- **`external/m1n1/src/hv.c`**: tried re-enabling `hv_arm_tick`
  after the printf removal, on the theory that the 1 kHz TX flood
  from the prints was what actually killed the Mac in 17 ms (not
  the FIQ handling path itself). Result: tick-enabled runs DO work
  now (guest runs fine, uptime returns non-zero), but the Mac still
  resets around 30-60 s regardless. Same timeline as tick-disabled
  — the HV tick is NOT the destabiliser after all, but also isn't
  helping. Reverted to tick-disabled for stability; left the gate
  conditional on `chip_id != T8132` so the next session can flip it
  once we find whatever IS causing the SMC reset.
- **`src/main.rs`**: added `cpuid` and `sha256 <text>` shell
  commands. Updated `help` listing.
- **`scripts/hv/batos_hv_interactive.py`**: stimulus parser uses
  `;;` instead of `|` as separator (dodges shell quoting weirdness),
  auto-appends `\r` if missing, also splits on newlines.
- **Docs**: added four evidence files:
    - `docs/2026-04-19_batos_under_hv_ttyACM2_boot_log.txt` — first
      clean boot log.
    - `docs/2026-04-19_batos_hv_interactive_help_session_extracted.txt`
      — first interactive `help` round-trip.
    - `docs/2026-04-19_batos_hv_full_demo.txt` — help / uname / mem /
      uptime / batfs full round-trip; `seconds: 52` uptime.
    - `docs/2026-04-19_batos_hv_cpuid_sha256.txt` — cpuid + sha256.

### Remaining for next session (priority order)

1. **Multi-minute sessions**. Still resets ~100 s in (same symptom
   with and without ticks). Suspect Apple SMC heartbeat. Concrete
   next experiment: trace what stock m1n1 does between chainloads
   — it clearly doesn't hit the reset, so something on its main
   `uartproxy_run` loop (`iodev_handle_events` + `iodev_read`) keeps
   SMC happy. Possibly the ON-BUS bus-master activity from DWC3 DMA
   is what pings SMC; the HV path has less bus activity between
   traps. Try a deliberate background bus-master from the HV (e.g.
   periodic memcpy between HV-owned pages to keep the coherence
   fabric active).
2. **`pyserial` opens to /dev/ttyACM2 still risk killing the HV** —
   the CDC SET_CTRL_LINE_STATE handler in DWC3 under HV may still
   hit an ungated IMPDEF MSR. Our workaround: use the `scripts/hv/
   batos_hv_interactive.py` path, which opens ttyACM2 ONCE and
   holds it, so the control messages fire once cleanly.
3. **AIC v3 Bat_OS driver** — we currently gate AIC init under HV,
   which means Bat_OS has no interrupts and can only poll. Fixing
   this unblocks Bat_OS's scheduler/timer work under HV.
4. **Remove the m1n1.macho chainload step**. Right now each session
   needs a fresh chainload. Ideally we persist the patched m1n1 via
   kmutil (same way stock m1n1 is installed) so Mac boot → patched
   m1n1 → automatic run_guest with Bat_OS payload.

---

## 2026-04-19 22:00 — Ubuntu — END-TO-END INTERACTIVE BAT_OS SHELL OVER USB-CDC ON M4 UNDER HV ✅✅

**Camera is now obsolete.** Typing `help\r` into the vuart CDC
endpoint from Ubuntu and receiving Bat_OS's actual kernel response
on the same port:

```
bat_os>
  help           — list commands
  uname          — kernel identity
  mem            — frame allocator stats
  fb             — framebuffer info
  uptime         — ticks since boot (CNTPCT_EL0 / CNTFRQ_EL0)
  batfs ls       — list BatFS files
  batfs create <name> <plaintext>
  batfs read <name>
  halt
bat_os>
```

Evidence: `docs/2026-04-19_batos_hv_interactive_help_session_extracted.txt`.

### The key delta from the 21:45 entry (which just had the prompt)

Ubuntu's tty layer is the enemy when you're trying to drive a serial
port transiently. `printf 'help\r' > /dev/ttyACM2` opens and closes
the tty, and that close momentarily drops DTR. m1n1's DWC3 CDC ACM
code only marks `dev->pipe[1].ready = true` when it sees
`SET_CTRL_LINE_STATE` with DTR set — so the write window is too
narrow for the OUT bulk endpoint to actually get armed and delivered.

Fix: open /dev/ttyACM2 from a **single long-lived Python process
that also drives the m1n1 HV proxy** (/dev/ttyACM1). One process
keeps DTR asserted the whole session, configures raw termios before
any I/O, and uses a reader/writer thread model that never closes
the tty between operations. New script:

- `scripts/hv/batos_hv_interactive.py` — drop-in replacement for
  `external/m1n1/proxyclient/tools/run_guest.py` that opens ttyACM2
  bidirectionally before starting the HV, spawns a reader thread,
  and (optionally) injects a canned command via the
  `BATOS_HV_STIMULUS` env var.

### Reproduce

```bash
cd /home/kaden-lee/code/Bat_OS
# Chainload patched m1n1 (same workflow as the 21:45 entry).

# Run the interactive script — injects 'help\r' after it sees the
# prompt. Merge ttyACM1 proxy traces + ttyACM2 vuart bytes in
# stdout; m1n1 traces are prefixed `TTY>`.
sg dialout -c "BATOS_HV_STIMULUS='help\\\\r' timeout 40 \
    /usr/bin/python3 scripts/hv/batos_hv_interactive.py" \
    | grep -v '^TTY> \[hv_exc'
# Filter the hv_exc_sync breadcrumbs (one per dockchannel MMIO trap)
# if you just want the Bat_OS output.

# For a fully interactive prompt, run without STIMULUS set and use
# a separate terminal pointed at /dev/ttyACM2 — but know that the
# pyserial-based path in the script keeps DTR asserted, which is the
# only way to get bytes to flow to m1n1 under HV.
```

### What's still hard for the next session

- **Mac still resets after ~30-60 s of HV runtime.** Suspect Apple
  SMC/AOP heartbeat. A persistent shell over multi-minute sessions
  needs either ticks-on + more gating of Apple IMPDEF MSRs in the
  FIQ path, or explicit pinging of whatever keeps SMC happy.
- **Opening ttyACM2 from an already-running second process kills the
  HV** (probably DTR-toggle-induced CDC control message handling
  under HV hits an unhandled IMPDEF MSR). Use a single driver
  process; don't try to `cat` the port while running the interactive
  script.
- **Echo loops appear if Ubuntu tty echo is on.** The script sets
  raw termios explicitly; don't fight it.

---

## 2026-04-19 21:45 — Ubuntu — Bat_OS BOOTS UNDER M1N1 HV — kernel log + `bat_os>` prompt on /dev/ttyACM2 ✅

**One-line status.** Bat_OS now runs as a guest under m1n1's
hypervisor on real M4 hardware. Full kernel boot banner, boot args,
microkernel init, BatFS init, and the `bat_os>` shell prompt all
stream over `/dev/ttyACM2` via a new dockchannel-UART vuart trap we
added to m1n1. Evidence saved at
`docs/2026-04-19_batos_under_hv_ttyACM2_boot_log.txt`.

### The big moves this session

**m1n1 HV (external/m1n1/src/hv.c + hv_exc.c):**

- Gated `hv_arm_tick(false)` behind `chip_id != T8132` — on M4 the
  FIQ handling path (hv_tick → hv_vuart_poll → aic_set_sw) hits
  AIC v3 state that destabilises within ~17 ms. Without the tick,
  the HV is idle at EL2 except when the guest traps. That sidesteps
  the reset entirely for normal guest operation.
- Added `iodev_console_flush()` immediately before `hv_enter_guest()`
  so all markers actually reach the host (no CNTP tick drives the
  async flush now).
- (Inherited from the prior session: AMX/VMKEY/SPRR/GXF MSR gates in
  hv_start; PMCR0/UPMC/IPI_SR/VM_TMR gates in hv_exc_entry/exit/fiq.)

**m1n1 dockchannel VUART trap — NEW (hv_vuart.c + proxy path + Python):**

- New `hv_map_vuart_dockchannel(base, iodev)` that `hv_map_hook`s the
  full 64 KiB dockchannel MMIO region with a handler that:
  - Traps DATA_TX8 writes → forwards the byte to `IODEV_USB_VUART`
    → surfaces on `/dev/ttyACM2`.
  - Returns a permanently-free TX FIFO (`TX_FREE = 0x100`).
  - Serves DATA_RX8 + DATA_RX_COUNT from the USB_VUART host→device
    ring (so host input can reach the guest when it gets through —
    see "known limitations" below).
  - Drains any stale RX bytes at setup time.
- New proxy op `P_HV_MAP_VUART_DOCKCHANNEL = 0xc11` wired through
  `proxy.c`, `proxy.h`, Python's `proxy.py`.
- `hv/__init__.py::map_vuart` now ALSO looks up
  `/arm-io/dockchannel-uart` in the ADT and calls the new op — on
  M4 this logs `Mapped dockchannel vuart at 0x388128000`.
- Offset-compute fix: the dockchannel register FIFO is at
  `base + 0x4014`. `base & 0xffff` masked bit 15 wrong on M4
  (the access address is 0x38812c014, `& 0xffff` yields 0xc014, not
  0x4014 — which sent DATA_TX_FREE to the default case, returning 0,
  which wedged Bat_OS's `while(read32(TX_FREE)==0)` forever). Handler
  now computes `addr - vuart_dc_base`. That was the decisive bug.

**Bat_OS (src/main.rs + src/arch/aarch64/apple/boot.s):**

- Detect HV (CurrentEL == EL1) at the top of `kernel_main_apple` and
  set an `under_hv` flag.
- Gate AIC + `bring_up_all()` behind `!under_hv` — on M4 the guest
  pass-through mapping of AIC v3 at 0x381000000 clashes with the
  configuration m1n1's HV already applied, triggering an L2C
  external error that crashes the HV. Under HV we just skip the
  hardware bring-up; Bat_OS has no IRQs yet anyway, and the shell
  polls the UART.
- Gate `soc::set_fb_info` behind `!under_hv` so every FB-touching
  path (`dcp::boot_splash`, `fb_console`, `apple_kernel_self_test`)
  auto-no-ops — prevents the 16 MiB FB paint from clobbering m1n1's
  freed framebuffer memory.
- Replace the `apple_serial_shell` inter-poll `wfe` with a
  `core::hint::spin_loop()` under HV. `wfe` blocks forever without
  CNTP ticks at EL1; the busy-poll lets `getc()` drive MMIO traps
  which drives `iodev_handle_events` which drains DWC3.
- Inherited from prior sub-session: `boot.s` already skips its
  16 MiB FB paint at EL1.

### What the new `/dev/ttyACM2` stream looks like (success path)

```
================================================
  BAT_OS — BARE METAL APPLE SILICON
  Running on REAL M4 hardware.
================================================

[boot] m1n1 handoff OK
  revision: 3
  machine_type: 0x00000000
  mem_size: 15419 MiB
  devtree: 540672 bytes
  ADT-resolved peripherals: 9 / 9
[boot] Initializing microkernel...
[initrd] no blob
  [mm] Frame allocator initialized — 15748512 KB free, heap @ 0x…
[boot] (HV guest) skipping AIC + hw bring-up
  (empty — dev fallback)
[boot] BatFS initialized (key=KDF(passphrase))
[boot] Initializing display...
[boot] No display — serial shell

bat_os>
```

665 bytes, clean, no echo garbage (see echo-gotcha below).

### Exact workflow to reproduce tonight

```bash
cd /home/kaden-lee/code/Bat_OS
# If you touched m1n1 or Bat_OS source:
#   cd external/m1n1 && make -j$(nproc) && cd -
#   bash build_apple.sh

# Wait for stock m1n1 (the Mac returns to stock after the HV session
# ends or is interrupted).
for i in $(seq 1 24); do
  [ -e /dev/ttyACM1 ] && udevadm info /dev/ttyACM1 | grep -q m1n1_uartproxy && break
  sleep 5
done
udevadm info /dev/ttyACM1 | grep ID_MODEL=   # expect bcee7f2

# Chainload the PATCHED m1n1. Absolute path is MANDATORY (the
# passwordless sudoers rule matches the absolute path exactly).
sudo -n --preserve-env=M1N1DEVICE,M1N1WAIT \
    M1N1DEVICE=/dev/ttyACM1 M1N1WAIT=1 \
    /usr/bin/python3 \
    /home/kaden-lee/code/Bat_OS/external/m1n1/proxyclient/tools/chainload.py \
    -S /home/kaden-lee/code/Bat_OS/external/m1n1/build/m1n1.macho
sleep 3
udevadm info /dev/ttyACM1 | grep ID_MODEL=   # expect "unknown" (our tag)

# BEFORE starting run_guest.py, start the vuart reader with ECHO OFF.
# Ubuntu's tty layer defaults to `echo echoctl icanon`, which means
# Bat_OS's TX bytes (including \r and \n) get ECHOED BACK as `^M` and
# `^J` sequences, which the Bat_OS shell then treats as input, which
# it echoes, which Ubuntu echoes, etc. Looks like repeating
# ^M^J=============^MM^J… — NOT a Bat_OS bug; it's tty echo.
rm -f /tmp/vuart.log
sg dialout -c 'stty -F /dev/ttyACM2 raw -echo -echoctl -icanon -icrnl -onlcr -opost min 1 time 0; nohup cat /dev/ttyACM2 > /tmp/vuart.log 2>/dev/null &'

# Now run the guest. (Python's hv.start() call will eventually hit
# SerialException when the HV eventually resets — that's fine, Bat_OS
# is running on the Mac under the HV regardless of Python's state.)
sg dialout -c "M1N1DEVICE=/dev/ttyACM1 timeout 30 /usr/bin/python3 \
    /home/kaden-lee/code/Bat_OS/external/m1n1/proxyclient/tools/run_guest.py \
    --raw --entry-point 0 /home/kaden-lee/code/Bat_OS/target/bat_os_apple.bin"

cat /tmp/vuart.log   # full Bat_OS kernel log up to the bat_os> prompt
```

### Known limitations (next-session targets)

1. **Ubuntu → Bat_OS input doesn't land.** Writing to /dev/ttyACM2
   via `printf`, `exec 3<>`, or pyserial does NOT appear in the
   guest's RX ring (`iodev_can_read(IODEV_USB_VUART)` always returns
   0). Hypothesis: when only `cat` (O_RDONLY) holds the port open,
   the USB CDC SET_CTRL_LINE_STATE DTR bit may not be set, so
   `dev->pipe[1].ready` stays false and the OUT EP isn't armed. Or
   the host-side TTY flush model is dropping the brief write window.
   - Try: open ttyACM2 via `exec 4<>/dev/ttyACM2` BEFORE cat, then
     `cat <&4` to read and `printf … >&4` to write, all over the
     same persistent fd.
   - Alternatively: drive ttyACM2 from inside run_guest.py — the same
     Python process has DWC3 access via proxy and can `iodev_write`
     / `iodev_read` directly without going through Ubuntu's tty stack.

2. **Mac eventually resets.** With `hv_arm_tick` disabled on M4, the
   HV is alive for ~30-60 s, then the Mac comes back as stock m1n1.
   Suspect Apple SMC/AOP heartbeat watchdog. For a persistent shell
   we need EITHER re-enable ticks + fix the remaining Apple-IMPDEF
   MSR causing the 17 ms reset, OR explicitly ping whatever the
   SMC expects.

3. **Opening /dev/ttyACM2 from pyserial kills the HV** — probably
   because DTR/RTS toggles trigger USB CDC control messages that
   m1n1's DWC3 handler processes under HV and hits an IMPDEF MSR
   or AIC write we haven't gated. Use only `cat` + printf for now;
   avoid pyserial / minicom / screen until we harden the CDC control
   path.

### Files changed this commit

- `external/m1n1/src/hv.c` — chip_id gate on hv_arm_tick,
  iodev_console_flush before eret.
- `external/m1n1/src/hv.h` — prototype for hv_map_vuart_dockchannel.
- `external/m1n1/src/hv_vuart.c` — new
  `handle_vuart_dockchannel` + `hv_map_vuart_dockchannel`.
- `external/m1n1/src/proxy.c` — P_HV_MAP_VUART_DOCKCHANNEL case.
- `external/m1n1/src/proxy.h` — P_HV_MAP_VUART_DOCKCHANNEL enum.
- `external/m1n1/proxyclient/m1n1/proxy.py` — matching Python enum
  + method.
- `external/m1n1/proxyclient/m1n1/hv/__init__.py` — `map_vuart()`
  also maps dockchannel on M4.
- `src/main.rs` — `under_hv` gate on AIC/bring_up/set_fb_info;
  `apple_serial_shell` uses `spin_loop` not `wfe` under HV.
- `docs/2026-04-19_batos_under_hv_ttyACM2_boot_log.txt` — evidence.

### Key realisation we learned the hard way

**Ubuntu tty `echo` is ON by default even for USB CDC.** The
`^M^J=============^MM^J` pattern that looked like a Bat_OS bug was
just Ubuntu echoing Bat_OS's CRLF output back to Bat_OS's shell,
which then echoed it back to Ubuntu, which echoed it back, etc. The
14-byte "banner" that looked like a truncated 48-char `=` banner was
actually Ubuntu's terminal printing the `\r` + `\n` from Bat_OS's
`uart::puts("\r\n")` as `^M^J` (via `echoctl`) — 4 chars — PLUS
Bat_OS's "================================================\n\r\n" chunked
weirdly by the echo loop.

Moral: `stty -F /dev/ttyACM2 raw -echo -echoctl -icanon -opost` is
mandatory before any interaction.

---

## 2026-04-19 20:45 — Ubuntu — m1n1 HV past hv_init + hv_start + eret; Mac USB resets ~17 ms into guest

**One-line status.** Guest now runs ~17 ms (seventeen 1 kHz HV timer
ticks) under the patched m1n1 hypervisor on real M4 hardware before
`/dev/ttyACM1` drops. HV itself is alive throughout — we see the new
`[hv_exc_fiq] enter` printf fire on every CNTP tick.

### What I gated this session

**m1n1 side (external/m1n1/src/):**

- `hv.c::hv_start` — gate the AMX/VMKEY/SPRR/GXF MRS reads that
  UNDEF on M4. Use `cpu_features->amx` (false on M4) for AMX_CTL_EL2
  / APVMKEYLO/HI_EL2 / APSTS_EL12, and `cpu_features->mmu_sprr`
  (false on M4) for SPRR_CONFIG_EL1 / GXF_CONFIG_EL1. Added
  `[hv_start] S0..S8` markers to match the `[hv_init] Mx` pattern.
- `hv.c::hv_init_secondary` — mirror the same gates on the write
  side (AMX/VMKEY/SPRR/GXF MSR writes).
- `hv_exc.c::hv_exc_entry` — skip `mrs(SYS_IMP_APL_PMCR0)` +
  `msr(...)` on M4 (`chip_id == T8132`). PMCR0 UNDEFs on M4, and
  the call fires on EVERY HV exception entry — without this gate,
  the very first CNTP tick post-eret triple-faults m1n1.
- `hv_exc.c::hv_exc_exit` — skip the matching PMCR0 restore.
- `hv_exc.c::hv_exc_fiq` — skip the PMCR0 / UPMCR0 / UPMSR /
  IPI_SR_EL1 block (all Apple IMPDEF).
- `hv_exc.c::hv_update_fiq` — skip the `SYS_IMP_APL_VM_TMR_FIQ_ENA_EL2`
  reg_set/reg_clr on M4 (IMPDEF timer-fiq virtualisation reg, UNDEFs).
- Added early `printf` breadcrumbs at the top of `hv_exc_sync`,
  `hv_exc_irq`, `hv_exc_fiq`, `hv_exc_serr` so we can see which
  kind of exception is firing from the stream of serial output.

**Bat_OS side (src/):**

- `arch/aarch64/apple/boot.s` — skip the 16 MiB framebuffer proof-
  of-life paint when `CurrentEL == EL1`. Under `run_guest.py`,
  Python calls `fb_shutdown(True)` which `free()`s the FB backing
  memory; stage-2 pass-through means writing the old FB physical
  address clobbers m1n1's own heap and the Mac hard-resets in a
  few ms. EL2 direct chainload still paints (camera verification).
- `main.rs::kernel_main_apple` — at entry, read `CurrentEL`; if
  EL1 (running under HV), skip `soc::set_fb_info(...)`. That makes
  every FB-consumer (`dcp::init_simple_fb`, `dcp::boot_splash`,
  `fb_console::init`, `fb_console::putc`) auto-no-op via their
  existing `fb_base() == 0` guards. Mem info is still populated.

### Where we are now

`run_guest.py --raw --entry-point 0 <any_binary>` with the patched
`external/m1n1/build/m1n1.macho` chainloaded:

- ✓ m1n1 proxy chainload succeeds (`udevadm` shows
  `m1n1_uartproxy_unknown`)
- ✓ `hv.init()` / page-table build / ADT fixup all run to completion
- ✓ `[hv_init] M0..M14` all print
- ✓ `[hv_start] S0..S8` all print
- ✓ `hv_enter_guest` eret's into the guest (no trap, no reset at
  eret)
- ✓ Guest executes (tested with a 2-instruction WFE-loop payload
  at `/tmp/wfe_guest.bin` — `d503205f; 17ffffff`)
- ✓ CNTP tick fires at 1 kHz, `[hv_exc_fiq] enter` prints on each
  tick for ~17 ticks (~17 ms)
- ✗ After ~17 ticks, `/dev/ttyACM1` drops (Python sees
  `SerialException: device reports readiness to read but returned
  no data`). `udevadm` post-crash shows the stock `bcee7f2` build
  back, i.e. the Mac rebooted.

Key fact: the HV is ALIVE during those 17 ms. The printfs demonstrate
m1n1 is still running at EL2 servicing the CNTP FIQ. So whatever kills
the machine happens AFTER the FIQ handler returns (ERET back to EL1
guest), and some number of cycles later we either:

(a) hit an Apple IMPDEF MSR in a code path I haven't gated yet
    (possibly the USB iodev handling path in `hv_tick`, or in
    `iodev_handle_events(uartproxy_iodev)` / `hv_vuart_poll()`),
(b) or we hit an Apple SMC/AOP heartbeat-watchdog that bites
    because m1n1 is spending all its cycles in FIQ and not pinging
    whatever keeps SMC happy,
(c) or the USB CDC TX ring in m1n1 stalls (IRQs masked during
    hv_exc_entry, DMA completions not being acked), USB hub
    decides device is dead, Mac USB host forcibly resets the
    port which cascades.

17 ms is suspicious — too short for a classic 30s Apple SMC watchdog,
too long for an immediate exception at eret. It's more consistent with
(a) or (c) — a USB-stall pattern fits the "TTY stream dies but no
exception printf" symptom.

### Exact workflow to reproduce tonight's state

```bash
cd /home/kaden-lee/code/Bat_OS

# m1n1 is already built; Bat_OS is already built. If you touched
# either, rebuild:
#   cd external/m1n1 && make -j$(nproc) && cd -
#   bash build_apple.sh

# Wait for the stock (bcee7f2) m1n1 to be live after the last reset
for i in $(seq 1 24); do
  [ -e /dev/ttyACM1 ] && udevadm info /dev/ttyACM1 | grep -q m1n1_uartproxy && break
  sleep 5
done
udevadm info /dev/ttyACM1 | grep ID_MODEL=   # expect bcee7f2

# Chainload the PATCHED m1n1 (absolute path — passwordless sudo
# rule in /etc/sudoers only matches the absolute path)
sudo -n --preserve-env=M1N1DEVICE,M1N1WAIT \
    M1N1DEVICE=/dev/ttyACM1 M1N1WAIT=1 \
    /usr/bin/python3 \
    /home/kaden-lee/code/Bat_OS/external/m1n1/proxyclient/tools/chainload.py \
    -S /home/kaden-lee/code/Bat_OS/external/m1n1/build/m1n1.macho
sleep 3
udevadm info /dev/ttyACM1 | grep ID_MODEL=   # expect unknown (our build tag)

# Smoke-test with the WFE loop (this is the MINIMAL guest — zero
# Bat_OS code in the path — isolates HV issues from Bat_OS issues):
sg dialout -c "M1N1DEVICE=/dev/ttyACM1 timeout 60 /usr/bin/python3 \
    /home/kaden-lee/code/Bat_OS/external/m1n1/proxyclient/tools/run_guest.py \
    --raw --entry-point 0 /tmp/wfe_guest.bin" 2>&1 | tee /tmp/hv.log

# Expect: [hv_init] M0..M14 (all), [hv_start] S0..S8 (all),
# ~17 × [hv_exc_fiq] enter, then SerialException.

# The same pattern reproduces with Bat_OS's bat_os_apple.bin payload —
# the guest just doesn't make it far enough to print anything before
# the USB dies, so to debug the HV itself use the WFE payload.
```

### Priority next moves (order of cheapest-experiments-first)

1. **Test with `hv_arm_tick` disabled on M4.** If we don't arm the
   CNTP tick, FIQ never fires. If the Mac stays alive indefinitely
   with no HV tick, then the HV FIQ path itself is the destabiliser.
   If the Mac STILL resets after ~17 ms even without FIQ, it's
   something else (SMC watchdog, USB idle timeout, etc.).
2. **If FIQ was the culprit** — audit everything `hv_tick`/
   `hv_vuart_poll`/`iodev_handle_events` does for more Apple IMPDEF
   MSRs. The chip_id-gate pattern is clear; just extend it.
3. **Add a `reg_set_sync` / `iodev_console_flush` call AT THE TOP**
   of `hv_exc_fiq` before the early printf. If the issue is TX
   buffer stall, seeing flush behavior change will tell us.
4. **Only once the Mac doesn't reset** — wire up a vuart for the
   M4 dockchannel UART (0x3_8812_8000). m1n1's existing vuart maps
   uart0 (0x3_ad20_0000, Samsung semantics). Bat_OS writes to
   dockchannel — different register layout. Either (A) patch
   `hv_vuart.c` to also recognise dockchannel register offsets and
   add a `hv_map_vuart_dockchannel(base, irq, iodev)` in
   `external/m1n1/src/` + Python `map_vuart_dockchannel` in
   `hv/__init__.py`, or (B) on M4 under HV have Bat_OS write to
   0x3_ad20_0000 with Samsung semantics (needs a new driver mode
   in `drivers/apple/uart.rs`). Option (A) is cleaner but requires
   knowing dockchannel reg semantics — we already have
   `external/m1n1/src/dockchannel_uart.c` upstream to copy from.

### Known gotchas I already hit so that next-Claude doesn't

- **`sudo -n /usr/bin/python3 external/m1n1/.../chainload.py`
  without absolute path** fails. The passwordless rule in
  `/etc/sudoers` matches `/usr/bin/python3 /home/kaden-lee/code/Bat_OS/external/m1n1/proxyclient/tools/chainload.py *`
  — relative path = password prompt = fail.
- **`/dev/ttyACM1` may disappear for 5-60 s** after a failed HV
  attempt while iBoot re-loads stock m1n1. The polling loop
  `for i in $(seq 1 24); do ... ; sleep 5; done` covers it.
- **Don't add long `udelay(...)` loops inside hv_start before
  `hv_enter_guest`.** I tried a 5 × 200 ms heartbeat diagnostic
  and it broke boot entirely (back to crashing at `S0`). Either
  `udelay` itself on M4 does something that destabilises the
  hardware when called repeatedly at EL2, or the extra 1-s delay
  trips an iBoot-side handoff timer. Short printfs are fine.
- **ttyACM0 is a USB hub on this host (IBP_Mini_Hub), not m1n1.**
  ttyACM1 is m1n1's proxy CDC endpoint (interface 00), ttyACM2 is
  m1n1's secondary CDC endpoint (interface 02) — that's where the
  vuart byte-stream will come out once dockchannel vuart is
  hooked up. Do NOT use `/dev/m1n1` — the symlink is present in
  `/etc/udev/rules.d/99-m1n1.rules` but the current kmutil-installed
  stock m1n1 uses different USB IDs than the udev rule expects.
- **`cd external/m1n1` persists across Bash calls.** This session's
  shell kept drifting to the m1n1 subdir. Always use absolute paths
  in commands (`/home/kaden-lee/code/Bat_OS/...`) rather than
  relative ones to dodge that.

### Files committed this session

- `external/m1n1/src/hv.c` — hv_start + hv_init_secondary gates;
  [hv_start] Sx markers; the diagnostic heartbeat loop was
  REMOVED before commit (it destabilised boot).
- `external/m1n1/src/hv_exc.c` — PMCR0 / UPMC / IPI_SR / VM_TMR
  gates on T8132; early printfs at each hv_exc_* entry.
- `src/arch/aarch64/apple/boot.s` — CurrentEL check, skip FB
  paint at EL1.
- `src/main.rs` — CurrentEL check, skip set_fb_info at EL1.

---

## 2026-04-19 20:15 — Ubuntu — m1n1 HV M4 bring-up partial; hangs inside hv_init

**Session-end handoff.** We pivoted from camera-pointed-at-screen
to the bigger play: make m1n1's hypervisor mode (`run_guest.py`)
work on M4 so m1n1 stays resident as hypervisor and forwards
guest-UART over USB-CDC — bidirectional interactive shell, no more
camera. The existing CLAUDE.md warning "Do NOT use run_guest.py on
M4" was right about the first trap (AMX_CONFIG_EL1 UNDEF) — we've
now gated all of those plus several more, and `run_guest.py`
progresses MUCH further, but hangs somewhere inside m1n1's C-side
`hv_init()`.

### Gates landed this sub-session

Four commits on top of 19:15's state:

- `61631102` — Python-side gates in `hv/__init__.py`:
  AMX_CONFIG_EL1 read+write, VMKEYLO/VMKEYHI/APSTS writes,
  SPRR_CONFIG_EL1/GXF_CONFIG_EL1 enable writes, secondary-CPU RVBAR
  loop, CPUSTART offset table — all skipped when MIDR PART is
  0x52 (M4 E-core) or 0x53 (M4 P-core). Plus `sysreg.py` gets new
  `MIDR_PART.T8132_DONAN_{ECORE,PCORE}` constants.
- `6ebdb34f` — `smp.c::smp_start_secondaries` adds `case T8132:`
  in the CPU_START_OFF switch (was falling through to "unknown"
  and returning early without setting `boot_cpu_idx`), plus an
  early `return` after `boot_cpu_idx` is set on M4 so the loop
  below doesn't P-cluster-RVBAR-SError the boot CPU.
- `79a30ff5` — `hv.c::hv_init` instrumented with `[hv_init] M0..M14`
  printf markers between every substep. Next session greps the
  serial log for the last marker to identify the trapping line.

### Where `run_guest.py` stands right now

`sg dialout -c "M1N1DEVICE=/dev/ttyACM1 /usr/bin/python3 external/m1n1/proxyclient/tools/run_guest.py --raw --entry-point 0 target/bat_os_apple.bin"`

- ✓ AMX skip
- ✓ VMKEY skip
- ✓ SPRR/GXF skip
- ✓ RVBAR skip (`Skipping secondary CPU RVBARs (M4 P-cluster SErrors)`)
- ✓ CPUSTART known (was "CPUSTART unknown for this SoC!", now silent)
- ✓ Page tables built, ADT uploaded, `Jumping to entrypoint at 0x…`
- ✗ **Hangs on `self.p.hv_init()` C-side.** Next session after
  chainloading the patched m1n1 will see the `[hv_init] Mx`
  markers and the LAST one printed before timeout is the one
  we need to gate.

### Chainloading the patched m1n1 — workflow that works

The patched m1n1 is at `external/m1n1/build/m1n1.macho` (built
locally; the kmutil-installed one in NVRAM is still the stock
`bcee7f2` build). To get the patched one running:

```bash
cd /home/kaden-lee/code/Bat_OS

# 1. (Re)build if you change any m1n1 source
cd external/m1n1 && make && cd -

# 2. Wait for stock m1n1 to be up after the last crash cycle
for i in $(seq 1 12); do
  [ -e /dev/ttyACM1 ] && udevadm info /dev/ttyACM1 | grep -q m1n1 && break
  sleep 5
done

# 3. Chainload the PATCHED m1n1 (no --raw — it's a Mach-O). The
#    -S flag skips the P-cluster RVBAR write that SErrors on M4.
sudo -n --preserve-env=M1N1DEVICE,M1N1WAIT \
    M1N1DEVICE=/dev/ttyACM1 M1N1WAIT=1 \
    /usr/bin/python3 external/m1n1/proxyclient/tools/chainload.py \
    -S external/m1n1/build/m1n1.macho

# Wait for "Proxy is alive again". udevadm should now show
#   ID_MODEL=m1n1_uartproxy_unknown  (our BUILD_TAG = "unknown")
# instead of `m1n1_uartproxy_bcee7f2` (stock).

# 4. Run the guest. sg wraps so the dialout group is effective
#    without needing sudo for /dev/ttyACM*. timeout keeps us
#    from hanging forever if m1n1 or the guest wedges.
sg dialout -c "M1N1DEVICE=/dev/ttyACM1 timeout 120 \
    /usr/bin/python3 external/m1n1/proxyclient/tools/run_guest.py \
    --raw --entry-point 0 target/bat_os_apple.bin" \
  2>&1 | tee /tmp/hv.log

# 5. Find the last hv_init marker:
grep "\[hv_init\]" /tmp/hv.log | tail -1
```

Expected progression after the last marker we see: identify the
trapping call, add a `chip_id == T8132` guard or skip the MSR,
rebuild, rechainload, re-run. 3–5 iterations should clear the
hv_init trap chain entirely.

### If the Mac's Apple watchdog bites between chainloads

Observed behavior: stock m1n1 hangs on the `m1n1_uartproxy_bcee7f2`
ID for up to 60 s after a failed HV attempt before the Mac resets
and comes back. Just poll `/dev/ttyACM1` until `udevadm info` shows
`m1n1_uartproxy_*`. No cold-cycle needed unless multiple failed
chainloads have put iBoot in a bad state (see earlier 17:35 entry —
which we now know was a linker foot-gun, not actual iBoot escalation,
so this note may never actually fire).

### Why this matters

Once hv_init clears and `run_guest.py` can actually boot Bat_OS as
a guest:

- m1n1 stays resident as hypervisor → USB-CDC endpoints stay alive
- `/dev/ttyACM1` forwards bytes to the guest Bat_OS's UART and
  vice versa
- Interactive shell from Ubuntu → `apple_serial_shell` on Bat_OS,
  zero work on the USB-CDC-in-Bat_OS front (which would otherwise
  be weeks of DWC3/DART/descriptor work)
- Camera goes away as a development bottleneck

---

## 2026-04-19 19:15 — Ubuntu — kernel self-test PASSES on M4 with on-screen output

**Milestone: Bat_OS is functionally operational along the post-splash
path on real M4 silicon.** Every LL/SC-on-Device rewrite this session
landed (rng::CTR, frame::alloc_frame, batfs::next_nonce, AIC stats,
heap UnsafeCell) is exercised under load and PASSes, with results
rendered in 2x-scaled text on the Mac's display.

### What's working end to end

Camera-verified at 19:12. Bat_OS runs through:
`_apple_start` → Rust → `mm::init` (frame + heap) →
process/scheduler/ipc/arch_exceptions init → AIC init →
`bring_up_all` (three DART bypasses) → `wdt::disable` →
`boot_args::parse` → ADT walk → auth init → BatFS init (with rng →
HMAC → SHA + AES + nonce) → `dcp::init_simple_fb` → `dcp::boot_splash`
(black bg + amber BAT_OS + cyan subtitle + dim footer) →
`fb_console::init` → `apple_kernel_self_test` (see below) →
`apple_serial_shell` idling on WFE.

Stable 40+ s per chainload before the standard Apple watchdog bites
(20–60 s window, doc'd in `M4_GROUND_TRUTH §2`).

### apple_kernel_self_test on-screen output

```
[boot] Splash rendered -- launching apple shell
[boot] FB console: uart mirror active

[selftest] starting kernel self-test
[selftest] frame::alloc_frame ... OK (addr=0x0000_0001_0xxx_xxxx)
[selftest]   free_frame returned
[selftest] batfs::create("selftest.txt") ... OK
[selftest] batfs::read+verify ... OK (43 B matched)
[selftest] batfs::create("notes.txt") ... OK
[selftest] batfs::stats = 2/128 files in use
[selftest] batfs::merkle_root = 0x........
[selftest] batfs::verify_all_integrity ... OK
[selftest] frame pool: N used / M total (... MiB free)
[selftest] all PASS

bat_os>
```

Every line is a real kernel call: frame allocator round-trip, two
BatFS creates (exercising NONCE_COUNTER increments), BatFS decrypt +
HMAC-SHA256 verify, file listing, Merkle-tree integrity check, and a
memory-pool status report. No faked output.

### Display / build hardening this session

Beyond the core LL/SC fixes:

- `dcp::argb8888_to_m4`: ARGB8888 → ARGB2101010 re-encoder, const-fn
  so all splash color literals stay authored in ARGB8888 for clarity
  but land in the M4 FB's native packing.
- `dcp::fill_screen`: `dsb sy` at the end so the 22 MiB wipe drains
  before subsequent draw_str calls (was leaving m1n1 boot-log text
  bleeding through the splash).
- `apple::uart::putc`: now mirrors every byte into `fb_console`, so
  char-level emitters (`print_num`, `puthex32`) show up on-screen
  instead of only the dockchannel MMIO.
- `fb_console`: 2x scaled rendering, row-copy scroll on overflow,
  cleanly below the splash.
- `font::draw_char_scaled` / `draw_str_scaled`: integer-block
  scaling, pure addition to the 8x16 API.
- `build_apple.sh`: refuses to ship a Linux-header binary (first
  4 bytes MUST be `0xf40300aa` = mov x20, x0 = `_apple_start`), so
  a plain `cargo build --release` slip can't make it to hardware.

### Where we are vs "fully operational"

Operational on the slow path (splash + self-test + silent shell).
Still missing for a genuinely full-featured kernel: timer IRQ +
preemptive scheduling (blocked on proper EL2 vectors + AIC routing),
MMU at EL2 (gate to proper process isolation + LSE atomics), USB-CDC
so Ubuntu can interactively drive the shell, and real process /
BatCave spawn. Those are each multi-day projects — see tasks
#19/#20/#23.

**Commits landed this session:**
- `ab0425e7` — `rng::CTR.fetch_add` → load+store
- `f7282171` — ARGB fix + frame/batfs LL/SC
- `f7a77b62` — `apple_serial_shell`
- `085afda5` — build_apple.sh safety check + linker foot-gun docs
- `7bcb0242` — fb_console + uart mirror
- `701bec8a` — AIC atomic stats non-atomic + self-test
- `05bc7c3a` — `dsb sy` fill_screen + row-copy scroll
- `c0e086ae` — scaled fb_console text
- `94b26f71` — mirror putc (not just puts) into fb_console

---

## 2026-04-19 18:40 — Ubuntu — splash FULLY verified; linker-script foot-gun found

**All of today's work now verified on real M4 hardware.** Camera at
18:40 shows the Bat_OS boot splash rendering stably for 90+ seconds
on the M4 display:

- Solid black background (ARGB2101010 constants correct).
- Amber `BAT_OS` title, cool-blue subtitle, dim-gray footer — all
  rendered via `dcp::boot_splash()` → `fill_screen` + `font::draw_str`.
- `ttyACM1/2` (m1n1 USB CDC) gone post-chainload → Bat_OS owns the
  Mac, no iBoot reset.

That means every LL/SC / ARGB / shell fix we landed today is on hot
code paths that executed cleanly: `mm::init` (non-atomic heap +
`reserve_range`), `batfs::init` (fixed `rng::CTR.fetch_add` and
`NONCE_COUNTER` load+store), `dcp::init_simple_fb`, `dcp::boot_splash`,
`apple_serial_shell` idling. All on commit `f7a77b62`.

### The foot-gun that cost an hour of bisecting

The "Mac-state iBoot reset loop" I documented earlier in this
session (journal 17:35) was **not** an Apple-firmware issue. Cold-
cycles and cable-cycles and clean macOS boots all failed to help
because my code was never the regression: my **build** was.

`.cargo/config.toml` sets
`rustflags = ["-C", "link-arg=-Tlinker.ld", ...]`, so a plain
`cargo build --release` links with the QEMU-virt linker script, which
places the 64-byte Linux kernel Image header (`b +0x40`, magic
`ARM\x64`, ...) at offset 0. `build_apple.sh` overrides with
`RUSTFLAGS="-C link-arg=-Tlinker_apple.ld"`, which places the Apple
stub `_apple_start` (`mov x20, x0`) at offset 0 instead. m1n1's
`chainload.py --raw --entry-point 0` jumps to offset 0 unconditionally.

I'd been running `cargo build --release` to iterate, which produced a
"valid-looking" binary whose first instruction was `b +0x40` —
chainload.py jumped into Linux-header code on the M4, faulted
immediately, and the Mac reset within ~2 seconds every time. The
same source tree built through `build_apple.sh` works; `cargo build
--release` does not. Once I re-ran `build_apple.sh`, the splash
rendered on the first chainload.

**Fix landed:** `build_apple.sh` now asserts the first four bytes of
`target/bat_os_apple.bin` decode to `mov x20, x0` (0xf40300aa LE),
and refuses to emit the binary if it sees the Linux-header opcode
(0x14000010 LE). It also picks up `rust-objcopy` from the rustup
toolchain dir if `rust-objcopy` isn't on `PATH`. No more silent
wrong-linker builds.

**Updated `docs/M4_GROUND_TRUTH.md §2`:** the "iBoot tightens under
repeated chainloads" entry is now redacted — that whole hypothesis
came from the wrong-linker red herring. The M4 actually tolerates
repeated chainloads fine.

**Open for next session:** now that the kernel runs stably on M4,
the real next work is teaching Bat_OS to own the USB-CDC endpoint
(so Ubuntu can read/write the `apple_serial_shell`), or any other
planned-OS direction. Pick whatever advances the roadmap.

**Files touched:** `build_apple.sh`, `docs/SESSION_JOURNAL.md`,
`docs/M4_GROUND_TRUTH.md`.

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
