# macOS-side MTP trace

Goal: capture the exact MTP mailbox sequence macOS runs so we can
replay it from m1n1 proxy and finally wake up the internal keyboard.
Our raw-proxy RE got stuck because a piece of the bring-up is inside
IOKit service matching that we can't reach from EL2; dtrace on a live
boot of macOS sees all of it.

Everything here runs **on the M4 booted into macOS**, not on the Ubuntu
host. Since our Bat_OS workflow always reboots into m1n1, you'll need
to reboot, pick "Macintosh HD" (or similar) at the m1n1 boot picker to
fall through to macOS, and run the steps below.

## One-time setup (once per Mac)

1. Reboot into Recovery (hold power button until "Loading recovery
   options"). Open Terminal → `csrutil enable --without dtrace`
   (partial SIP — only dtrace relaxed; everything else stays
   protected). Exit, reboot into macOS.

2. In macOS Terminal, verify dtrace can probe the kernel:
   ```
   sudo dtrace -ln 'fbt::_ZN14AppleASCWrapV6*:entry' | head
   ```
   Should list the AppleASCWrapV6 probe functions. If not, something
   about SIP is still blocking — check `csrutil status`.

## Capture a trace

1. Connect USB-C to the Ubuntu host (so power is steady — don't want
   an unscheduled sleep).

2. In Terminal:
   ```
   sudo dtrace -q -s /path/to/Bat_OS/scripts/macos/trace_mtp_mailbox.d \
       -o ~/mtp_init.trace
   ```
   Leave running.

3. Trigger an MTP re-init. Two options:

   - **Sleep/wake** (preferred — close lid for ~3s, open): MTP
     firmware re-runs its Hello sequence. Trace captures the whole
     mailbox dance.
   - **Kextunload/load** of `com.apple.driver.AppleA7IOP-ASCWrap-v6`
     if the runtime allows it (likely won't — it's a built-in kext).
     Skip this if it errors.

4. After wake, interact with the keyboard briefly so HID traffic
   flows. Then `Ctrl-C` the dtrace process.

5. Copy `~/mtp_init.trace` to the Ubuntu host (scp, iCloud, USB
   thumb drive — whatever works). Put it somewhere this repo can see
   it.

## Feed it back to the Ubuntu side

On the Ubuntu side:
```
python3 scripts/macos/parse_dtrace_trace.py path/to/mtp_init.trace \
    > scripts/hv/replay_mtp_sequence.py
```

That parser produces a Python replay script that reruns the exact
mailbox sequence under our m1n1 proxy. Chainload patched m1n1, then:
```
sg dialout -c 'M1N1DEVICE=/dev/ttyACM1 python3 \
    scripts/hv/replay_mtp_sequence.py'
```

If MTP Hellos in response, internal keyboard is one small step away
(DockChannel HID subscribe already exists in `batos_hv_interactive.py
_mtp_kbd_probe`).

## If the trace is empty or sparse

- FBT probes require SIP-dtrace-relaxed. Re-check `csrutil status`.
- Some Apple-kernel symbols only appear after reboot. Try again.
- Apple may ship a DEVELOPMENT kernelcache with more symbols. Check
  `/System/Library/KernelCollections/` for a `development` or `debug`
  flavor, switch to it via:
  ```
  sudo nvram boot-args="kcsuffix=development"
  sudo kmutil configure-boot ...
  ```
  (leave this for a follow-up if fbt doesn't see enough symbols).

## Why the full XNU HV-trace approach was abandoned

See `docs/SESSION_JOURNAL.md` (2026-04-21/22 entries). Short version:
booting the macOS 26.3 kernelcache under m1n1 HV gets through several
layers (register convention, SP setup, PAC enable) but hits a wall at
APIA-key-authenticated function-pointer dispatch. Fixing that requires
either SEP cooperation (impossible) or re-signing
`LC_DYLD_CHAINED_FIXUPS` entries with a chosen key (1-2 weeks of
focused PAC work).

dtrace on the live kernel skips all that — we just watch macOS do its
own job on its own hardware.
