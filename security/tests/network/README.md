# Bat_OS network fuzz harness

Scripts here exercise specific ATTACK-NET-* findings in
`../../PENTEST_NETWORK.md`. They assume:

* Bat_OS running in QEMU (see repo root `Makefile`'s `run` target).
* Python 3.10+ with `scapy` (`pip install scapy`). Root or
  `setcap cap_net_raw=eip $(which python3)` is required for raw sockets
  for the L2 scripts.

## Two QEMU network modes

### A. User-mode NAT (default)

```
qemu-system-aarch64 ... -netdev user,id=n0,net=10.0.2.0/24,hostfwd=tcp::2222-:2222 \
                        -device virtio-net-device,netdev=n0
```

In this mode the **host can reach Bat_OS at 10.0.2.15 ONLY through the
hostfwd redirection**. You can send IP/TCP/UDP to `127.0.0.1:2222`
(which Bat_OS sees on port 2222). L2 tricks (ARP poison, malformed
Ethernet) are invisible because QEMU translates them. Most of the
scripts here therefore talk to `127.0.0.1:<hostfwd port>` and rely on
Bat_OS initiating a TCP connection to a remote host that we control.

### B. Tap bridge (needed for L2 attacks)

```
sudo ip tuntap add dev tap0 mode tap user $USER
sudo ip link set tap0 up
sudo ip addr add 10.0.2.1/24 dev tap0
qemu-system-aarch64 ... -netdev tap,id=n0,ifname=tap0,script=no,downscript=no \
                        -device virtio-net-device,netdev=n0
```

This exposes a raw L2 between host (10.0.2.1) and guest (10.0.2.15).
`fuzz_arp.py`, `fuzz_ip.py`, and the off-path TCP-injection scripts
require this mode.

## Running

```bash
sudo python3 fuzz_arp.py        # ATTACK-NET-001, 002
sudo python3 fuzz_ip.py         # 004, 005, 008
sudo python3 fuzz_tcp.py        # 009, 010, 011, 019, 022
python3 fuzz_dns.py             # 035, 036, 039, 041 (needs Bat_OS to call resolve())
python3 fuzz_tls.py 4443        # 026, 027, 028, 029, 030 — runs as TLS-1.3 decoy server
python3 fuzz_http.py 8080       # 042, 045, 046 — runs as malicious HTTP server
```

Each script prints "SENT <attack-id>" and waits. Watch the Bat_OS UART
log (QEMU `-serial stdio`) for `[fw] BLOCKED ...`, a panic, or silent
acceptance. Acceptance ≈ vuln confirmed.

## What each script asserts

| Script | Attack IDs | What it does |
|--------|------------|--------------|
| fuzz_arp.py | 001, 002 | Injects gratuitous ARP replies claiming to be 10.0.2.2. Then, checks by sending a ping from 10.0.2.1; if Bat_OS MACs its reply to the poisoned MAC, win. |
| fuzz_ip.py | 004, 005, 008 | Crafts IPv4 frames with IHL<5, IHL>total_len, and wrong checksum. Watch for kernel panic (ATTACK-005). |
| fuzz_tcp.py | 009, 010, 011, 019, 022 | Predicts Bat_OS ISN (deterministic), then injects blind RST, blind FIN, blind data. |
| fuzz_dns.py | 035, 036, 039, 041 | Sends spoofed DNS responses to UDP :12345 — wrong TXID, unsolicited, NXDOMAIN-with-answer, etc. |
| fuzz_tls.py | 026, 027, 028, 029, 030 | Minimal TLS-1.3 responder that sends pathological ServerHello variants (TLS 1.2 legacy, zero key_share, wrong Finished, garbage GCM tag). |
| fuzz_http.py | 042, 045, 046 | HTTP server that sends slowly, header-bombs, or injects Location: CRLF. |

## Expected hardening check

When the Top-10 fixes in PENTEST_NETWORK.md §5 are applied, re-running
these scripts should all produce `REJECTED` on the Bat_OS UART (firewall
drop, TLS alert, DNS mismatch, etc.) and *no* kernel panics.
