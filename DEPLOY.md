# Bat_OS — Deployment Guide: Real M4 MacBook

> **STUMP #137 update.** The original DEPLOY.md was written before the
> M4 reverse-engineering work and described an M1-era flow. Following
> it on M4 would silently brick your day-1 (`run_guest.py` writes
> `AMX_CONFIG_EL1`, which traps on M4 because M4 has SME instead of
> AMX). This rewrite reflects the verified-on-real-hardware path. Cross-
> reference `docs/M4_GROUND_TRUTH.md` and `CLAUDE.md` if anything here
> looks off.

## What you need

- Your **M4 MacBook 14"** (Mac16,1 / J604 / T8132 "Donan") — the target
- A second computer **running Linux** (any modern distro). **NOT
  Windows** — m1n1's composite USB device doesn't enumerate on
  Windows without a vendor INF that nobody publishes. Macs work too,
  but Ubuntu is what we test on.
- A USB-C cable that supports data (not charge-only)
- Python 3 + `pyserial` on the dev machine

## Overview

```
┌─────────────┐    USB-C cable    ┌─────────────────┐
│ Dev Machine │◄─────────────────►│  M4 MacBook     │
│  (Linux)    │   serial debug    │  (running m1n1) │
│             │                   │                 │
│ Sends:      │                   │ Receives:       │
│ bat_os ELF  │ ────────────────► │ Loads + runs    │
│             │                   │ Bat_OS!         │
└─────────────┘                   └─────────────────┘
```

## Step 1: Install m1n1 on the M4 (in macOS Recovery)

The Asahi Linux installer (`alx.sh`) **refuses to run on M4** as of
April 2026 — Asahi support hasn't landed for this generation yet. We
install m1n1 directly via Apple's `kmutil` tool from Recovery.

1. Boot the M4 into Recovery: hold the power button while powering on
   until "Loading startup options" appears. Select **Options**.
2. From Recovery's Utilities menu, open **Terminal**.
3. Reduce security on the macOS volume so a non-Apple kernel image
   can boot:

   ```bash
   csrutil disable                    # SIP off
   bputil -nkc                        # full security → permissive
   ```

4. Bless m1n1 as the boot payload:

   ```bash
   kmutil configure-boot \
       -c /path/to/m1n1.macho \
       -v /Volumes/<your-volume>
   ```

   You can drop a built m1n1 onto a USB stick first if Recovery
   networking is unavailable. The `external/m1n1/build/` directory
   in this repo has a known-good build for M4 (with the `--skip-
   secondary-cpus` patch already applied).

5. Reboot. Holding the power button now shows a boot picker with
   "m1n1" as one option.

## Step 2: Use the vendored m1n1 (don't `pip install`)

This repo ships m1n1 at `external/m1n1/` with the M4-specific
P-cluster SError fix (`--skip-secondary-cpus` / `-S`) **already
patched into `chainload.py`**. Use this copy. Do NOT install m1n1
from upstream — that one will hang the M4 P-cores at
`run_secondary` because it doesn't have the `-S` workaround.

```bash
cd Bat_OS
# m1n1 lives at external/m1n1 already; nothing to fetch.
# Install Python deps:
pip3 install pyserial construct
```

## Step 3: Connect the machines

1. Plug a USB-C cable from the dev machine into the M4. **Use the
   left port** (closest to MagSafe) — some ports don't expose the
   debug device.
2. On the M4: hold the power button → boot picker → select **m1n1**.
3. The screen goes black — that's normal, m1n1 is running headless.

## Step 4: Verify the serial link

On the dev machine, look for the m1n1 USB device:

```bash
# Linux
ls /dev/ttyACM*               # should show /dev/ttyACM0 or similar

# macOS
ls /dev/tty.usbmodem*
```

Test the connection:

```bash
python3 external/m1n1/proxyclient/tools/shell.py
```

You should land in the m1n1 Python shell. If it hangs, double-check
the cable + port and the boot picker selection.

## Step 5: Build + chainload Bat_OS

From the **build machine** (any system that can run `cargo build`,
typically the Mac):

```bash
make build
# Produces:
#   target/aarch64-unknown-none/release/bat_os       (ELF)
#   target/aarch64-unknown-none/release/bat_os.bin   (flat Image)
```

Copy `bat_os.bin` to the **dev machine** (the one connected to the
M4). Then chainload:

```bash
python3 external/m1n1/proxyclient/tools/chainload.py \
    /path/to/bat_os.bin
# (the -S / --skip-secondary-cpus flag is pre-applied in our vendored
#  copy — see CLAUDE.md for why)
```

This sends Bat_OS over USB to m1n1, which jumps to it. You should
see on the dev machine's terminal:

```
================================================
  BAT_OS — BARE METAL APPLE SILICON
  Running on REAL M4 hardware.
================================================
```

And on the MacBook's screen: the Bat_OS auth gate.

> **DO NOT use `run_guest.py`.** `run_guest.py` initializes a
> hypervisor that writes `AMX_CONFIG_EL1`, which traps on M4 (no AMX
> on M4 — Apple replaced it with SME on this generation). The
> M1/M2-era guides all say `run_guest.py`; ignore them. Always use
> `chainload.py`.

## Step 6: Get back to macOS

m1n1 doesn't auto-fall-through to macOS. To go back:

1. Hold the power button for 10 seconds to force-shutdown.
2. Hold power again until "Loading startup options" appears.
3. Pick **Macintosh HD** (or your macOS volume).

## Step 7: Production setup (still WIP)

The "no dev machine" path doesn't exist yet — m1n1 auto-boot from
the EFI partition on M4 needs more work (boot.conf, payload
signature handling, the kmutil chain). For now every boot is a
chainload from the dev machine.

When this lands, it'll look like:

```
Power Button → Apple iBoot → m1n1 (auto-boot) → Bat_OS auth gate
```

Tracking issue: see `docs/SESSION_JOURNAL.md` for the latest.

## Boot chain (current, real)

```
Power Button
    │
    ▼
Apple iBoot (ROM)
    │
    ▼
macOS boot picker  (hold power button)
    │
    ▼
m1n1 (chainloaded by kmutil configure-boot)
    │
    ▼
chainload.py over USB pushes bat_os.bin
    │
    ▼
Bat_OS auth gate (passphrase, optional YubiKey [planned])
    │
    ▼
Bat_OS desktop
```

## Troubleshooting

### "No serial device found"

- Use the **LEFT** USB-C port on the M4 (closest to MagSafe).
- Cable must support data, not just charging.
- On Linux you may need `usermod -aG dialout $USER` + relogin to
  read `/dev/ttyACM0` without sudo.
- On Windows: it won't work at all without a vendor INF Apple
  doesn't publish — switch to Linux. See `UBUNTU_QUICKSTART.md`.

### "m1n1 boots but `chainload.py` hangs"

- You probably installed upstream m1n1 instead of the vendored copy.
  The upstream `chainload.py` doesn't have `-S` and the M4 P-cluster
  SErrors on the RVBAR write. Use `external/m1n1/proxyclient/tools/
  chainload.py` from this repo.

### "Bat_OS crashes immediately"

- Check the serial output. Bat_OS prints a panic with the offending
  EL/PC.
- `docs/DEBUGGING_RUNBOOK.md` has known failure modes and fixes.
- M4 MMIO addresses are NOT the same as M1's. Many references on
  the Asahi wiki are M1-era and look plausible on M4 but aren't.
  Always cross-check `docs/M4_GROUND_TRUTH.md`.

### "I want to develop in QEMU instead"

- See `QUICKSTART.md`. `make render-live` boots Bat_OS under
  QEMU/HVF on the Mac with virtio-gpu, virtio-keyboard,
  virtio-tablet, and (since STUMP #136) virtio-blk for persistent
  BatFS. Way faster iteration than M4 chainload while developing.

## Security notes

- m1n1 has a USB serial debug interface. This is a development feature
  — in a "production" deployment you'd want to either disable m1n1
  USB or physically airgap the M4. Today: development-mode only.
- The auth gate runs BEFORE any persistent data is decrypted (the
  passphrase derives the BatFS master key).
- Failure modes:
  1. Wrong passphrase 5 times → all keys destroyed (`security/auth.rs`
     MAX_ATTEMPTS).
  2. Duress passphrase → silent wipe (`security/wipe.rs::Duress`).
  3. No passphrase / deadman timer expires → wipe.
- Secure Enclave integration (the docs claim "master keys never touch
  RAM") is **not yet implemented** — `drivers/apple/sep.rs` doesn't
  exist. Master key currently lives in `static mut MASTER_KEY` per
  `src/fs/batfs.rs`. SEP work is a future STUMP. Don't trust the
  "government-grade" framing literally until SEP lands.
