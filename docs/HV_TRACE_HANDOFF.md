# HV-Trace Handoff — MTP/AOP Internal Keyboard on M4

**Goal**: Get the M4 MacBook's internal keyboard working in Sphragis by
booting the MTP (Multi-Touch Processor) firmware and subscribing to
its HID events.

**Status at handoff**: Raw proxy RE exhausted. MTP FW runs its own
state machine post-RUN=1 but never writes its first Hello to OUTBOX.
SMC (same `iop,ascwrap-v6` driver) Hellos reliably. Difference is
IOKit service-layer setup only visible via live macOS trace.

**You are working on**: `feat/js-engine-browser-posix` branch.

---

## Quick orientation (read in this order)

1. `docs/SESSION_JOURNAL.md` — the 2026-04-22 entries (newest first)
   document the full arc. The 03:30, 04:45, 05:30, and `a113067e`
   commit message summarize exactly what fails.
2. `scripts/hv/boot_mtp_dartmap.py` — the most complete MTP boot
   attempt. Does everything we know how to do. Fails.
3. `scripts/hv/sphragis_hv_interactive.py` `_mtp_kbd_probe` (line 303) —
   the full keyboard pipeline including DockChannel + MTPProtocol.
   Use this once MTP Hellos.
4. `macos_dump/` — Apple kexts extracted from J604 macOS 26.3
   kernelcache. `AppleA7IOP-ASCWrap-v6.macho` + `AppleA7IOP.macho` are
   the drivers we're trying to mimic. Also has `ioreg_ASCWrapV6.txt`
   showing live state during working macOS boot.

## What's already proven

- Mailbox layout: INBOX at `+0x8800`, OUTBOX at `+0x8830` (from
  `AppleASCWrapV6::_inbox/_outbox` disasm — see `/tmp/io.txt` or
  `scripts/re/find_aop_init.py` for how to regenerate).
- Doorbell: write `0x10` to `+0x1004` then `0x1` to `+0x1014` (from
  `_triggerFiqNmi` disasm).
- First-message kick: `mgmt.start()` sending `SetIOPPower(STATE=0x220)`
  = type 6 at bits 59:52, state 0x220.
- DART mapping works: `dart.iomap_at(stream, iova, phys, size)` for
  each segment — verified installed but didn't unblock MTP.
- m1n1 WDT fix in `external/m1n1/src/wdt.c` — safe variant. Clears
  only `0x3882BC224` deadline-arm bit. Earlier variant corrupted SMC.
- SMC reproducibly boots from fresh power cycle. Use this as your
  "is my proxy alive" sanity check.

## What HV-trace needs to accomplish

Capture every MMIO read/write to these ranges during macOS's native
MTP init, with source PC:

- `0x394600000..0x394688000` — MTP ASC reg[0]
- `0x394050000..0x394054000` — MTP ASC reg[1] (small)
- `0x394800000..0x39480c000` — dart-mtp reg[0]
- `0x394810000..0x394814000` — dart-mtp reg[1]
- `0x394b00000..0x394b38000` — dockchannel-mtp regs
- Optionally `0x390600000..0x390688000` — AOP (same pattern)

Goal: diff this sequence against `scripts/hv/boot_mtp_dartmap.py`.
The delta is the missing setup.

## Paths to try (in order)

### 1. Patch `run_guest.py` to boot the J604 kernelcache

We have the decompressed Mach-O FILESET at
`macos_dump/kernelcache.mac16j.bin` (120MB). The straightforward
attempt:

```bash
cd /home/kaden-lee/code/Sphragis/external/m1n1/proxyclient
# Need to install trace_mtp.py to load MTP tracer config
# (adapt from existing hv/trace_aop.py — they share the ASC tracer class)
python3 tools/run_guest.py \
    -m hv/trace_mtp.py \
    -c "hv.trace_range(irange(*u.adt['/arm-io/mtp'].get_reg(0)))" \
    -c "hv.trace_range(irange(*u.adt['/arm-io/dart-mtp'].get_reg(0)))" \
    /home/kaden-lee/code/Sphragis/macos_dump/kernelcache.mac16j.bin
```

**Expect this to fail** on first try — the kernelcache won't have
what it needs to probe AFPS, find a root volume, etc. But the failure
mode will tell you exactly what's missing.

### 2. Glue for macOS boot

What macOS needs beyond what `run_guest.py` provides:

- `boot-args` matching what iBoot passes. Grab these live from macOS:
  `ssh kadenlee@kadens-MacBook-Pro.local 'nvram -p | grep boot-args'`
  and `ioreg -l | grep -E 'boot-args|BootArgs' | head`.
- SEP ticket. Apple verifies the kernelcache signature via SEP during
  boot. May need to bypass via m1n1's SEP-passthrough or accept that
  SEP verification runs against iBoot's ticket in memory already.
- APFS container access. macOS reads root from /dev/disk. In HV we
  could try virtio-blk to present our host's APFS partition, but
  that's a big project. **Simpler approach**: aim to get MTP init
  working BEFORE filesystem init. macOS's `AppleA7IOP::start()`
  happens during IOKit service matching in early kernel, before the
  filesystem is mounted. You just need the MMIO writes during that
  window.

### 3. Fallback — use Asahi's macOS-guest plumbing

Asahi Linux's m1n1 fork (`upstream/asahi` branch) has more developed
macOS-guest support. Check:
```bash
git -C external/m1n1 log --all --grep='macOS.*guest\|xnu guest\|boot macOS'
```
Cherry-pick anything relevant. Their `macos_experiments/` tree (if
present) may have the boot glue.

### 4. When MTP Hellos under HV trace

Once macOS boots far enough that MTP attaches (you'll see
`AppleASCWrapV6` printed in HV log), the trace log will have
everything. Save it:

```python
hv.set_logfile(pathlib.Path("/tmp/mtp_trace.log").open("w"))
```

Then post-process: filter lines with MTP base addresses, sort by
timestamp, extract writes-only. That's your target sequence.

## Known gotchas

- **Power cycles are EXPENSIVE**. Each failed attempt poisons MTP/AOP
  state until power-cycle. Plan each run to get max info in one shot.
- **DON'T use our patched m1n1's `chainload` if booting stock macOS**
  — the AP WDT disable happens at kmutil-installed m1n1 boot time.
  Our chainload re-runs it, which broke SMC in previous tests. Start
  from a fresh boot (stock kmutil m1n1) and either a) do your work
  within 118s before AP WDT fires, or b) `p.write32(0x3882BC224, 0)`
  manually from Python before the timeout.
- **Don't write `0xffffffff` to `0x3882B8008`/`0x3882B802C`/`0x3882B8020`**.
  Those are SMC panic-scratch regs in the same page. Corrupts SMC for
  the rest of the boot.
- `dapf_init_all()` hangs on M4 when iterating dart-mtp. Use targeted
  `dapf_init("/arm-io/dart-aop")` instead, and skip dapf for dart-mtp.
- **Don't probe AOP+0x8200 or SMC+0x44 from cold** — both trigger DAPF
  SYNC in m1n1. Read +0x40 and +0x48 are fine.
- `adt.region_base` doesn't exist on `/arm-io/mtp/iop-mtp-nub`. Use
  `get_reg(0)` or parse `reg` property manually.
- MTP firmware blob at `firmware/mtp/J604_MtpFirmware.bin` has an
  `rkosftab` wrapper. Find the Mach-O via `.find(b'\xcf\xfa\xed\xfe')`.

## Success criteria

1. `scripts/hv/boot_mtp_dartmap.py` prints `*** MTP BOOT OK ***`
   followed by `*** KEYBOARD INITIALIZED ***`.
2. Pressing keys on the M4's internal keyboard causes `[mtp-kbd] HID
   report: XX XX XX...` lines in the script's stderr.
3. Running with `SPHRAGIS_HV_MTP_BRIDGE_TO_VUART=1` causes the keystrokes
   to appear on Sphragis's vuart — letting Sphragis demos use the
   internal keyboard.

## Minimum first-run command (once setup works)

```bash
# Fresh power cycle required
sg dialout -c 'PYTHONUNBUFFERED=1 /usr/bin/python3 scripts/hv/boot_mtp_dartmap.py' | tee /tmp/mtp_attempt.log
```

## If you get stuck

Worth-a-try shortcuts before going full HV:

- **Try MTP with SMCClient-pattern endpoint start**: don't use raw
  `StandardASC` — make a class like `MTPClient(StandardASC)` that
  subclasses and explicitly starts endpoints before mgmt.start.
- **Try swapping SMC and MTP order**: we've only done SMC-then-MTP.
  Maybe MTP needs to come first (weird, but cheap to test).
- **Try `hv.init()` then proxy-only**: m1n1's HV init does extra
  AIC/interrupt setup that the raw proxy path skips. Even without
  booting a guest, `hv.init()` side effects might enable what FW
  needs.

Good luck. You have everything the prior session built. Don't redo
proxy-side experiments — we've truly exhausted them.

---

## End state for this handoff

Branch: `feat/js-engine-browser-posix` @ `a113067e`
Last commit: "MTP: DART segment mappings don't help either — full
matrix exhausted"

m1n1 rebuild status: `external/m1n1/build/m1n1.macho` has the safe
WDT fix (commit `3a72b48b`).

Sphragis demo loop: unaffected, external USB keyboard works.
