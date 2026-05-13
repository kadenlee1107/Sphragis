#!/usr/bin/env python3
"""Real-peer-interop smoke for WireGuard — gap-audit item 043.

Proves that Bat_OS's WireGuard outbound path actually traverses
virtio-net to a real listener on the host. The script:

  1. Binds a UDP socket to 0.0.0.0:<wg-port> on the host.
  2. Boots Bat_OS in QEMU virt with default user-mode networking
     (gateway 10.0.2.2 maps to host loopback).
  3. Runs `wg-test-outbound 10.0.2.2:<wg-port>` at the shell. This
     registers a fresh fake peer, sets its endpoint to the host
     listener, then calls `wg_dispatch::initiate_connect` — which
     builds a real WG Init via the IPC mailbox and hands it to
     `udp::send`.
  4. Asserts the host UDP socket receives a datagram whose first
     byte is the WireGuard Init message-type (0x01) and whose
     total length matches the Phase-2 framing for an Init packet
     (148 bytes: type + reserved + sender_index + ephemeral_pk +
     enc_static + enc_timestamp + mac1 + mac2).

This validates the integration we already prove in-process via
`wg-dispatch-selftest` / `wg-initiator-e2e-selftest` — but over
the actual NIC tx ring + ARP + Ethernet + IP + UDP stack. No
Python crypto is needed: the Init wire bytes are deterministic
and self-validating.

Pass: exit 0. Fail: exit non-zero with serial log path.
"""
from __future__ import annotations

import socket
import sys
import time
from datetime import datetime
from pathlib import Path

import pexpect

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
LOG = (
    ROOT
    / f"logs/qemu-tests/wg-real-peer-e2e-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
)
LOG.parent.mkdir(parents=True, exist_ok=True)

# Pick a high port to avoid colliding with anything privileged-only.
# 51820 is WG's standard port; QEMU user-net SLIRP forwards traffic
# destined to 10.0.2.2:<port> through to the host's loopback.
WG_PORT = 51820

QEMU_ARGS = [
    "qemu-system-aarch64",
    "-machine", "virt",
    "-cpu", "max",
    "-m", "2G",
    "-display", "none",
    "-netdev", "user,id=net0",
    "-device", "virtio-net-device,netdev=net0",
    "-serial", "mon:stdio",
    "-kernel", str(KERNEL),
]


WG_INIT_MSG_TYPE = 0x01
WG_INIT_MSG_LEN = 148


def main() -> int:
    if not KERNEL.exists():
        print(f"[wg-real-peer] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    # Bind the host listener first so QEMU's SLIRP has somewhere to
    # forward the packet to.
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    try:
        sock.bind(("0.0.0.0", WG_PORT))
    except OSError as e:
        print(f"[wg-real-peer] cannot bind UDP {WG_PORT}: {e}", file=sys.stderr)
        return 2
    sock.settimeout(15.0)

    print(f"[wg-real-peer] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"bat_os > ", timeout=90)
        time.sleep(0.5)

        c.sendline(f"wg-test-outbound 10.0.2.2:{WG_PORT}")
        # Wait for the in-kernel report that udp::send queued the
        # Init on the tx ring. If this fails we know the kernel
        # side never got far enough to put bytes on the wire.
        idx = c.expect([
            rb"WG-OUTBOUND-SENT",
            rb"\xe2\x9c\x97 \S+",
        ], timeout=20)
        if idx == 1:
            print("[wg-real-peer] FAIL — kernel reported error before tx", file=sys.stderr)
            print(f"[wg-real-peer] log: {LOG}", file=sys.stderr)
            return 1
        print("[wg-real-peer] kernel reports Init queued on tx ring")

        # Now wait for the bytes to actually land on the host
        # listener via SLIRP's user-net NAT.
        try:
            data, peer = sock.recvfrom(2048)
        except socket.timeout:
            print("[wg-real-peer] FAIL — no UDP datagram arrived on host", file=sys.stderr)
            print(f"[wg-real-peer]        SLIRP forwarding to 10.0.2.2:{WG_PORT} may be filtered.", file=sys.stderr)
            print(f"[wg-real-peer] log: {LOG}", file=sys.stderr)
            return 1

        print(f"[wg-real-peer] received {len(data)} bytes from {peer}")

        if len(data) != WG_INIT_MSG_LEN:
            print(
                f"[wg-real-peer] FAIL — wrong length: {len(data)} (expected {WG_INIT_MSG_LEN})",
                file=sys.stderr,
            )
            return 1
        if data[0] != WG_INIT_MSG_TYPE:
            print(
                f"[wg-real-peer] FAIL — wrong message type: 0x{data[0]:02x} (expected 0x{WG_INIT_MSG_TYPE:02x})",
                file=sys.stderr,
            )
            return 1
        if data[1:4] != b"\x00\x00\x00":
            print(
                f"[wg-real-peer] FAIL — reserved bytes not zero: {data[1:4].hex()}",
                file=sys.stderr,
            )
            return 1

        # mac1 = bytes 116..132 (16 B BLAKE2s keyed MAC). Must be
        # non-zero (a fresh handshake mixes responder pubkey + the
        # rest of the message into it).
        mac1 = data[116:132]
        if mac1 == b"\x00" * 16:
            print("[wg-real-peer] FAIL — mac1 is all-zero", file=sys.stderr)
            return 1

        print("[wg-real-peer] PASS — WG Init traversed virtio-net to a real host listener")
        print(f"[wg-real-peer]   first byte: 0x{data[0]:02x}  reserved: {data[1:4].hex()}")
        print(f"[wg-real-peer]   sender_index: 0x{int.from_bytes(data[4:8], 'little'):08x}")
        print(f"[wg-real-peer]   mac1 (16 B):  {mac1.hex()}")
        print(f"[wg-real-peer] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[wg-real-peer] FAIL — timeout in shell expect", file=sys.stderr)
        print(f"[wg-real-peer] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        sock.close()
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())
