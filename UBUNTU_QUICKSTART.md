# Bat_OS — Ubuntu Live-USB Quickstart

Because Ubuntu live USB is tmpfs (RAM-only), every power loss wipes the
environment. This file is the paste-and-go ritual to get back to
"chainload works" in under 5 minutes.

## What to transfer from the Mac to Ubuntu

Copy these two things over (USB drive, scp, whatever):

1. `external/m1n1/` — the whole tree (already pre-patched with `-S`)
2. `target/bat_os_apple.bin` — latest build

## On Ubuntu, one-time setup per boot

```bash
# 1. Install the three things Ubuntu live USB doesn't have by default
sudo apt update
sudo apt install -y python3-construct python3-serial gcc-aarch64-linux-gnu

# 2. Allow your user to use the serial device without sudo (optional —
#    we still use sudo for the actual run but this is cleaner)
sudo usermod -a -G dialout $USER && newgrp dialout

# 3. cd to the m1n1 proxyclient directory
cd ~/m1n1/proxyclient    # or wherever you copied external/m1n1 to
```

## Boot the Mac into m1n1

1. Power off the Mac
2. Hold power button → "Loading startup options" → Options → Continue
3. Actually, if m1n1 is set as boot object (it is), just turn it on
   normally. Should auto-boot into m1n1 and reach "Running proxy..."

## Chainload Bat_OS

```bash
sudo M1N1DEVICE=/dev/ttyACM0 python3 tools/chainload.py \
    --raw --entry-point 0 -S \
    /path/to/bat_os_apple.bin
```

The `-S` flag is critical on M4 — skips the P-cluster RVBAR writes
that SError. (Already baked into our vendored m1n1 copy in
`external/m1n1/proxyclient/tools/chainload.py` — see the
`--skip-secondary-cpus` flag.)

## If `/dev/ttyACM0` doesn't exist

Check with:
```bash
ls /dev/ttyACM* /dev/ttyUSB*
dmesg | tail -30        # look for "cdc_acm" or "ttyACM" lines
```

Possible ACM numbers: ttyACM0, ttyACM1 (the composite device exposes
two serial interfaces). The one connected to m1n1's proxy is typically
ACM0 or ACM1. Try both if the first doesn't work.

## Common failure modes

| Symptom | Cause | Fix |
|---|---|---|
| `could not open port /dev/m1n1: [Errno 2]` | Missing `M1N1DEVICE` env var | Add `M1N1DEVICE=/dev/ttyACM0` |
| `Permission denied` on the serial port | Not in dialout group | `sudo` the command, or add user to dialout + re-login |
| `ModuleNotFoundError: No module named 'construct'` | apt packages not installed | `sudo apt install python3-construct python3-serial` |
| `... executing as aarch64-linux-gnu-as not found` | gcc-aarch64 missing | `sudo apt install gcc-aarch64-linux-gnu` |
| Mac spontaneously reboots during chainload | m1n1 RVBAR SError on M4 | Use `-S` flag |
| Mac resets at jump to payload | Wrong payload format / entry | Confirm with `file bat_os_apple.bin` says `data` (NOT `Linux kernel ARM64 Image`) |
| Stuck in exception loop | Previous crash, serial link dead | Power-cycle the Mac, wait for "Running proxy..." |

## Current known issue on M4

m1n1 outputs warnings about unsupported chip versions:
```
MCC: Unsupported version:mcc,t8132
cpufreq: Chip 0x8132 is unsupported
```

These are non-fatal (safe defaults used). But they explain why some
paths like `run_guest.py` (hypervisor mode — tries to MSR AMX_CONFIG_EL1
which doesn't exist on M4) crash. **Use `chainload.py`, not `run_guest.py`.**
