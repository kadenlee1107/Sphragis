# Sphragis infrastructure inventory

The machines, IPs, hostnames, and access patterns. Updated whenever a
machine is added or its address changes. Both Mac Claude and Ubuntu
Claude should read this before any cross-machine command.

## Machines

### Mac M4 (the target)
- **Model:** Mac16,1 / J604 (Apple M4 MacBook Pro 14")
- **macOS user:** `kadenlee`
- **Repo path:** `/Users/kadenlee/Sphragis/`
- **Role:** Sphragis target hardware. Mac Claude runs here when in macOS.
- **Boot states:**
  - macOS → Mac Claude active, can build/edit/commit
  - m1n1 → Sphragis chainload target. Mac Claude unavailable.

### Ubuntu dev box (the proxy host)
- **Hostname:** `kaden-lee-AMD-Ryzen-7-8700F-8-Core-Processor`
- **Tailscale IP:** `100.70.246.39`
- **Linux user:** `kaden-lee` (verified via SSH 2026-04-18)
- **Repo path:** `/home/kaden-lee/code/Sphragis/`
- **Kernel:** Linux 6.17.0-20-generic x86_64
- **Hardware:** AMD Ryzen 7 8700F 8-Core
- **Role:** Persistent dev host. Drives m1n1 chainload via /dev/m1n1
  (or /dev/ttyACM0). Ubuntu Claude runs here.
- **SSH from Mac works:** `ssh kaden-lee@100.70.246.39 ...`
  Mac's `~/.ssh/id_ed25519.pub` is in
  `/home/kaden-lee/.ssh/authorized_keys`.
- **Note:** `/dev/ttyACM0` exists even when Mac is NOT in m1n1 — some
  other USB CDC device is plugged in. When Mac IS in m1n1, m1n1 will
  appear as `/dev/m1n1` (via the udev rule) or as a second ACM
  number. Auto-detection in `scripts/chainload.sh` handles this.

## Tailscale

Both machines on Tailscale (private VPN, free tier).
- Mac: TBD — needs `brew install tailscale && sudo tailscale up`
- Ubuntu: connected as `100.70.246.39`

Once Mac is on Tailscale too, Mac Claude can do things like:
```bash
ssh kaden@100.70.246.39 './scripts/chainload.sh'
ssh kaden@100.70.246.39 'tail -f ~/code/Sphragis/logs/chainload-latest.log'
scp target/sphragis_apple.bin kaden@100.70.246.39:~/code/Sphragis/target/
```

## GitHub

- **Repo:** https://github.com/kadenlee1107/Sphragis (private)
- **Default branch:** `feat/js-engine-browser-posix`
- **Auth:** SSH (kaden's `~/.ssh/id_ed25519` on each machine)
- **Both Claudes push:** signed by their respective machine. Commits
  show "Co-Authored-By: Claude" lines.

## When in doubt

Run on Mac:
```bash
hostname; whoami; pwd; tailscale ip -4 2>/dev/null
```

Run on Ubuntu:
```bash
hostname; whoami; pwd; tailscale ip -4
```

Update this file if anything changed.
