# Bat_OS infrastructure inventory

The machines, IPs, hostnames, and access patterns. Updated whenever a
machine is added or its address changes. Both Mac Claude and Ubuntu
Claude should read this before any cross-machine command.

## Machines

### Mac M4 (the target)
- **Model:** Mac16,1 (Apple M4 MacBook Air)
- **macOS user:** `kadenlee`
- **Repo path:** `/Users/kadenlee/Bat_OS/`
- **Role:** Bat_OS target hardware. Mac Claude runs here when in macOS.
- **Boot states:**
  - macOS → Mac Claude active, can build/edit/commit
  - m1n1 → Bat_OS chainload target. Mac Claude unavailable.

### Ubuntu dev box (the proxy host)
- **Hostname:** `kaden-lee-AMD-Ryzen-7-8700F-8-Core-Processor`
- **Tailscale IP:** `100.70.246.39`
- **Linux user:** `kaden` (assumed; verify with `whoami`)
- **Repo path:** `~/code/Bat_OS/` (assumed; user can `pwd` to confirm)
- **Hardware:** AMD Ryzen 7 8700F 8-Core
- **Role:** Persistent dev host. Drives m1n1 chainload via /dev/m1n1
  (or /dev/ttyACM0). Ubuntu Claude runs here.

## Tailscale

Both machines on Tailscale (private VPN, free tier).
- Mac: TBD — needs `brew install tailscale && sudo tailscale up`
- Ubuntu: connected as `100.70.246.39`

Once Mac is on Tailscale too, Mac Claude can do things like:
```bash
ssh kaden@100.70.246.39 './scripts/chainload.sh'
ssh kaden@100.70.246.39 'tail -f ~/code/Bat_OS/logs/chainload-latest.log'
scp target/bat_os_apple.bin kaden@100.70.246.39:~/code/Bat_OS/target/
```

## GitHub

- **Repo:** https://github.com/kadenlee1107/Bat_OS (private)
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
