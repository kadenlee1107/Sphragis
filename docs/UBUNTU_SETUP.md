# Ubuntu persistent-host setup (one-time, ~2-3 hours)

This is the "graduate from live USB" setup. After this, Ubuntu is
your permanent Sphragis dev host — survives power loss, boots in 30s,
Claude Code installed, Tailscale connected to the Mac, everything.

## 0. Choose the install target

**Option A: External USB SSD (recommended)**
- Need: a USB SSD, 128 GB+. USB-3 speeds or faster.
- Pros: doesn't touch your Windows PC's internal drive, portable,
  fast reinstall if you ever wreck it.
- Cons: costs $30-50 if you don't have one lying around.

**Option B: Dual-boot the Windows PC**
- Need: 60+ GB free on the Windows drive, 30 min for safe partitioning.
- Pros: no extra hardware.
- Cons: shrinks Windows partition (reversible but fiddly), some risk
  if partition math goes wrong.

Steps below assume **Option A** (SSD). For dual-boot, the steps are
nearly identical; at the Ubuntu installer "Installation type" screen
pick "Install alongside Windows" instead of "Erase disk and install".

## 1. Make the installer USB

On any computer:
1. Download Ubuntu 24.04 LTS ISO from ubuntu.com/download/desktop.
2. Use **balenaEtcher** or **Rufus** (Windows) / **Raspberry Pi Imager**
   (mac/linux) to burn the ISO to a USB stick (different from the SSD
   you're installing TO).
3. Safely eject.

## 2. Install Ubuntu to the SSD

1. Plug the **installer USB** into your Windows PC.
2. Plug the **target SSD** into another USB port.
3. Reboot, interrupt boot to enter the boot menu (F12 / F11 / F2
   depending on BIOS).
4. Pick the installer USB.
5. At the Ubuntu welcome screen, click **Install Ubuntu**.
6. Language, keyboard, and network as normal.
7. **Installation type:** pick **Erase disk and install Ubuntu**
   (the target SSD must already be selected; DOUBLE-CHECK you're
   not picking the Windows drive).
8. Timezone, username (probably `kaden`), password, done.
9. When prompted, **remove the installer USB** and reboot.
10. Keep the SSD plugged in. In the boot menu (F12/F11), pick the
    SSD. Ubuntu boots fresh.

## 3. First-boot setup script

Clone Sphragis and run the setup script, which installs every package
we need:

```bash
# Set up SSH key for GitHub (one-time)
ssh-keygen -t ed25519 -C "kaden-ubuntu"
cat ~/.ssh/id_ed25519.pub
# Copy that output. On github.com, Settings → SSH Keys → Add.

# Clone the repo
mkdir -p ~/code && cd ~/code
git clone git@github.com:kadenlee1107/Sphragis.git
cd Sphragis

# Run the setup script
chmod +x scripts/*.sh
./scripts/setup.sh
```

The setup script installs:
- `python3-construct python3-serial` — m1n1 proxyclient deps
- `gcc-aarch64-linux-gnu` — asm stub for chainload
- `openssh-server` — so Mac Claude can SSH in
- `tailscale` — private VPN to the Mac
- `ffmpeg v4l-utils` — Elgato screen capture
- `tmux` — persistent sessions so serial doesn't die on SSH disconnect
- Rust nightly + `rust-src` + `rustup` — for `cargo build` on Ubuntu
- `aarch64-unknown-none-softfloat` target — for cross-compiling Sphragis

It also sets up:
- Your user in the `dialout` group (for `/dev/ttyACM*` access without sudo)
- A udev rule for m1n1's VID/PID so the serial device gets consistent naming

## 4. Tailscale (10 min, makes remote driving possible)

On Ubuntu:
```bash
sudo tailscale up
# Follow the URL to auth. Use your GitHub login.
tailscale ip -4   # note the 100.x.y.z address
hostname          # note the machine name
```

On Mac:
```bash
brew install tailscale
sudo tailscale up
# Same auth. Both machines now share a private network.

# Test SSH from Mac to Ubuntu:
ssh <ubuntu-username>@<ubuntu-hostname>
```

Once this works, Mac Claude can drive Ubuntu commands remotely via
Tailscale SSH. No port forwarding, no firewall rules, zero config.

## 5. Install Claude Code on Ubuntu

Instructions change over time — go to claude.ai/code for the current
installer. Roughly:
```bash
curl -fsSL https://claude.ai/install.sh | sh
claude --version
```

Then inside the Sphragis folder, run `claude`. It'll read `CLAUDE.md`
automatically on first session start.

## 6. First test (without M4, sanity-check the env)

```bash
cd ~/code/Sphragis
./scripts/rebuild.sh --check    # cargo check, should succeed
```

If that works, your Ubuntu env is ready. Next time you boot the Mac
into m1n1, you can run:
```bash
./scripts/chainload.sh target/sphragis_apple.bin
```

and it Just Works.

## 7. Optional: Elgato screen capture setup

If you have an Elgato capture card (HD60 S / HD60 X / Cam Link):
1. Plug the Elgato USB into the Ubuntu PC.
2. Check it enumerates: `ls /dev/video*` should show `/dev/video0`
   (or higher index).
3. Test capture:
   ```bash
   ffmpeg -f v4l2 -i /dev/video0 -frames:v 1 /tmp/test.png
   ```
4. For M4 HDMI out: Mac16,1 (M4 MacBook Pro 14") has an HDMI port on the right side; plug it into the Elgato directly. (Earlier notes in this doc said MBA / no-HDMI — that was wrong, target machine is an MBP.)
   USB-C to HDMI adapter. Plug Mac USB-C → HDMI cable → Elgato HDMI in.
5. Boot Mac into m1n1 and see if the bat logo appears on the captured
   video. If yes, Elgato can capture Sphragis too. If no, bare-metal
   Sphragis doesn't currently drive HDMI out — we'd need to port DCP
   external-display routing before this works.

## 8. When power goes out

Unplug, plug back in, boot to Ubuntu. Everything survives:
- Your code (on the SSD)
- Python packages, apt packages (on the SSD)
- Tailscale connection (auto-reconnects)
- Rust toolchain (on the SSD)
- Your GitHub auth
- `~/.ssh` keys

Only thing lost: anything you wrote that wasn't committed. So: commit
frequently. Ubuntu Claude should `git push` at the end of every
session per the journal convention.
