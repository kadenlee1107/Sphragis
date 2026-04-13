# Bat_OS — Deployment Guide: Real M4 MacBook

## What You Need

- Your M4 MacBook (the target)
- A second computer with USB-C (the dev machine — any Mac, PC, or Linux box)
- A USB-C cable connecting the two
- Python 3 on the dev machine

## Overview

```
┌─────────────┐    USB-C cable    ┌─────────────────┐
│ Dev Machine  │◄────────────────►│  M4 MacBook      │
│ (any PC/Mac) │   serial debug   │  (running m1n1)  │
│              │                  │                  │
│ Sends:       │                  │ Receives:        │
│ bat_os.bin   │ ──────────────►  │ Loads + runs     │
│              │                  │ Bat_OS!          │
└─────────────┘                   └─────────────────┘
```

## Step 1: Install m1n1 on Your M4 MacBook

m1n1 is the Asahi Linux bootloader. It creates a new boot volume
alongside macOS — your macOS stays untouched.

From macOS Terminal on the M4 MacBook, run:

```bash
curl https://alx.sh | sh
```

This runs the Asahi Linux installer. When prompted:
- Choose "m1n1 only (expert)" — we don't want Linux, just the bootloader
- Allocate minimal disk space (1GB is enough)
- Follow the prompts to completion

After installation, you'll have a new boot option in macOS Startup Disk.

## Step 2: Set Up the Dev Machine

On your second computer (the one you'll control m1n1 from):

```bash
# Clone m1n1
git clone --recursive https://github.com/AsahiLinux/m1n1.git
cd m1n1

# Install Python dependencies
pip3 install pyserial construct serial
```

## Step 3: Connect the Machines

1. Plug a USB-C cable from the dev machine to the M4 MacBook
2. On the M4 MacBook: reboot and hold the power button to enter Startup Manager
3. Select the "m1n1" boot volume
4. The MacBook screen may go black — that's normal, m1n1 is running

## Step 4: Verify Connection

On the dev machine, check that m1n1's serial device appeared:

```bash
# macOS dev machine:
ls /dev/tty.usbmodem*

# Linux dev machine:
ls /dev/ttyACM*
```

You should see a new serial device. Test the connection:

```bash
python3 proxyclient/tools/shell.py
```

If you see the m1n1 shell prompt, the connection is working.

## Step 5: Deploy Bat_OS

Copy `bat_os_apple.bin` from your build machine to the dev machine,
then run:

```bash
# From the m1n1 directory on the dev machine:
python3 proxyclient/tools/run_guest.py /path/to/bat_os_apple.bin
```

This sends the Bat_OS binary over USB to m1n1, which loads it into
memory and jumps to it.

You should see on the dev machine's terminal:
```
Sending payload...
Running guest at 0x810000000...

================================================
  BAT_OS — BARE METAL APPLE SILICON
  Running on REAL M4 hardware.
================================================
```

And on the MacBook's screen: the Bat_OS auth gate.

## Step 6: Production Setup (No Dev Machine Needed)

Once Bat_OS is working, you can eliminate the dev machine entirely:

### Option A: USB Boot
1. Put `bat_os_apple.bin` on a USB drive
2. Configure m1n1 to auto-load from USB on boot
3. Power on → m1n1 → loads from USB → Bat_OS

### Option B: Partition Boot
1. Write `bat_os_apple.bin` to the allocated boot partition
2. Configure m1n1 to chainload from that partition
3. Power on → m1n1 → Bat_OS (no USB needed)

### m1n1 Auto-Boot Configuration

Create a file on the EFI partition:

```
# /m1n1/boot.conf
payload=/bat_os_apple.bin
```

Or set via m1n1 shell:
```python
# In m1n1 proxy shell:
p.smp_start_secondaries()
u.mmu_shutdown()
p.kboot_raw(open('bat_os_apple.bin', 'rb').read())
```

## Boot Chain (Production)

```
Power Button
    │
    ▼
Apple iBoot (ROM)
    │
    ▼
m1n1 bootloader (auto-boot mode)
    │  Loads bat_os_apple.bin
    ▼
Bat_OS Auth Gate
    │  Passphrase + YubiKey
    ▼
Bat_OS Desktop
```

No macOS. No second machine. No network.
Just you and the passphrase.

## Troubleshooting

### "No serial device found"
- Try a different USB-C port (some ports don't support debug)
- Use the LEFT USB-C port on the MacBook (closest to MagSafe)
- Make sure the cable supports data (not charge-only)

### "m1n1 not booting"
- Hold power button for 10 seconds to enter recovery
- Re-select the m1n1 boot volume
- May need to re-install m1n1 if macOS updated firmware

### "Bat_OS crashes immediately"
- Check serial output for panic messages
- MMIO addresses may differ on your specific M4 revision
- Run `python3 proxyclient/tools/shell.py` to probe hardware

## Security Notes

- m1n1 has a USB serial debug interface — this is a development feature
- In production, disable m1n1's USB debug mode
- The auth gate runs BEFORE any data is decrypted
- If someone steals the MacBook and boots it:
  1. They see the passphrase prompt
  2. Wrong password 5 times → all keys destroyed
  3. Duress code → fake boot + silent wipe
  4. No passphrase → dead man's switch expires → wipe
