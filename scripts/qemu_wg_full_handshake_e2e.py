#!/usr/bin/env python3
"""Closes the WireGuard handshake loop over real virtio-net.

`qemu_wg_real_peer_e2e.py` validates that Sphragis's WG Init reaches
a host UDP listener with valid Phase-2 framing. This test takes
the next step: a Python Noise IK responder (`wg_responder.py`)
decrypts the Init, builds a Response, and sends it back through
SLIRP. Sphragis processes the Response via `dispatch_response`,
upgrades the session's `their_sender_index` from 0 to the
responder's index, and the `wg-test-outbound` shell command
prints `WG-SESSION-ESTABLISHED`.

End-to-end proof that Sphragis's Noise IK implementation interops
with an external responder over the wire — closes the full
gap-audit item 043 entry.

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
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG = (
    ROOT
    / f"logs/qemu-tests/wg-full-handshake-e2e-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
)
LOG.parent.mkdir(parents=True, exist_ok=True)

# Make the responder importable when this script lives in scripts/.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import wg_responder as wgr

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


def main() -> int:
    if not KERNEL.exists():
        print(f"[wg-full] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    # ── Generate the responder keypair the kernel needs to use ──
    resp_sk, resp_pk = wgr.x25519_keypair()
    resp_pk_hex = resp_pk.hex()
    print(f"[wg-full] responder pubkey: {resp_pk_hex}")

    # ── Bind host UDP listener BEFORE booting QEMU ──
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    try:
        sock.bind(("0.0.0.0", WG_PORT))
    except OSError as e:
        print(f"[wg-full] cannot bind UDP {WG_PORT}: {e}", file=sys.stderr)
        return 2
    sock.settimeout(15.0)

    print(f"[wg-full] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"sphragis > ", timeout=90)
        time.sleep(0.5)

        # Kick off the handshake with the responder's pubkey.
        c.sendline(f"wg-test-outbound 10.0.2.2:{WG_PORT} {resp_pk_hex}")

        # Wait for the kernel's "Init queued" marker.
        idx = c.expect([rb"WG-OUTBOUND-SENT", rb"\xe2\x9c\x97 \S+"], timeout=20)
        if idx == 1:
            print("[wg-full] FAIL — kernel reported error before tx", file=sys.stderr)
            print(f"[wg-full] log: {LOG}", file=sys.stderr)
            return 1
        print("[wg-full] kernel reports Init queued on tx ring")

        # ── Receive Init from Sphragis over SLIRP ──
        try:
            init_bytes, peer = sock.recvfrom(2048)
        except socket.timeout:
            print("[wg-full] FAIL — no Init datagram arrived on host", file=sys.stderr)
            return 1
        print(f"[wg-full] received {len(init_bytes)}-byte Init from {peer}")

        # ── Decrypt + build Response ──
        try:
            sender_index, eph_pk, init_static_pk, ck, hh = wgr.process_init(
                init_bytes, resp_sk, resp_pk,
            )
        except Exception as e:
            print(f"[wg-full] FAIL — process_init: {e}", file=sys.stderr)
            return 1
        print(f"[wg-full]   init.sender_index = 0x{sender_index:08x}")
        print(f"[wg-full]   init.eph_pk       = {eph_pk.hex()[:16]}...")
        print(f"[wg-full]   init.static_pk    = {init_static_pk.hex()[:16]}...")
        print("[wg-full]   mac1 + enc_static + enc_timestamp all verified")

        my_idx = wgr.random_sender_index()
        try:
            response, _resp_eph_sk = wgr.build_response(
                sender_index, eph_pk, init_static_pk, ck, hh, my_idx,
            )
        except Exception as e:
            print(f"[wg-full] FAIL — build_response: {e}", file=sys.stderr)
            return 1
        print(f"[wg-full] built {len(response)}-byte Response (my_idx 0x{my_idx:08x})")

        # Send Response back to Sphragis. peer == (host-side src) the
        # SLIRP NAT chose to represent the guest; sendto that 4-tuple
        # so SLIRP routes it back into the guest as if from the
        # original dst.
        sock.sendto(response, peer)
        print(f"[wg-full] Response sent back to {peer}")

        # ── Sphragis should see Response, mark session Established ──
        idx = c.expect([
            rb"WG-SESSION-ESTABLISHED",
            rb"no Response within deadline",
            rb"\xe2\x9c\x97 \S+",
        ], timeout=15)
        if idx != 0:
            print("[wg-full] FAIL — kernel did not mark session Established", file=sys.stderr)
            print(f"[wg-full] log: {LOG}", file=sys.stderr)
            return 1
        print("[wg-full] PASS — full Noise IK handshake completed over the wire")
        print(f"[wg-full] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[wg-full] FAIL — timeout in shell expect", file=sys.stderr)
        print(f"[wg-full] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        sock.close()
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())
