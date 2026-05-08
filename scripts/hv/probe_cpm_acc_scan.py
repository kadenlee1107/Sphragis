#!/usr/bin/env python3
"""Brute-force scan of CPM (Cluster Performance Manager) and ACC
(Apple CPU Complex) MMIO ranges to find counters that tick.

Two snapshots ~60 s apart, diff. Any address whose 32-bit value
changed between the two snapshots is a candidate timer.

CPM range: 0x210e40000 + 0xc010 (E-cluster CPM)
ACC range: 0x210f00000 + 0x40088 (E-cluster ACC)

We scan a sample of addresses (every 4 bytes for the first 0x400,
then every 16 bytes for the rest) to keep total reads tractable
over USB serial proxy.

Run while Mac is at stock m1n1:
  sg dialout -c "/usr/bin/python3 scripts/hv/probe_cpm_acc_scan.py"

If reads SError, the proxy will hang and we'll see no diff output.
"""
import sys
import os
import time
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))
os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")

from m1n1.proxy import *
from m1n1.proxyutils import *
from m1n1.utils import *


def gen_offsets():
    """Pack of offsets to scan within each block.
    Skip 0x40-0x6f — found to SError repeatedly on CPM."""
    offs = []
    # First 0x40 — every 4 bytes (this range survived prior probes)
    offs += list(range(0x000, 0x040, 4))
    # Skip 0x40-0x70 — known-SError range
    offs += list(range(0x070, 0x100, 4))
    # Sparse mid-range — every 16 bytes
    offs += list(range(0x100, 0x400, 16))
    return offs


def scan_block(p, base, name):
    out = {}
    bad = 0
    for off in gen_offsets():
        try:
            out[(name, off)] = p.read32(base + off)
        except Exception:
            bad += 1
            out[(name, off)] = None
            # If we get too many SErrors in a row, the proxy's dead
            if bad > 20:
                # Liveness check
                try:
                    p.iodev_whoami()
                except Exception:
                    print(f"  {name} proxy died at off=0x{off:x}", flush=True)
                    return out
    return out


def main():
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    ProxyUtils(p, heap_size=128 * 1024 * 1024)
    print("[scan] proxy alive", flush=True)

    blocks = [
        (0x210e40000, "ECPU_CPM"),
        (0x210f00000, "ECPU_ACC"),
        # Skip P-cluster — PCPU MMIO SErrors per HAS_GUARDED_IO_FILTER.
    ]

    print("[scan] snapshot 1 starting", flush=True)
    snap1 = {}
    for base, name in blocks:
        print(f"[scan] scanning {name} @ {base:#x}…", flush=True)
        result = scan_block(p, base, name)
        snap1.update(result)
    print(f"[scan] snapshot 1 done — {len(snap1)} reads", flush=True)

    sleep_s = int(os.environ.get("SCAN_SLEEP_S", "15"))
    print(f"[scan] sleeping {sleep_s} s…", flush=True)
    time.sleep(sleep_s)

    print("[scan] snapshot 2 starting", flush=True)
    snap2 = {}
    for base, name in blocks:
        print(f"[scan] scanning {name} @ {base:#x}…", flush=True)
        result = scan_block(p, base, name)
        snap2.update(result)
    print(f"[scan] snapshot 2 done — {len(snap2)} reads", flush=True)

    print()
    print("=== addresses that CHANGED between snap1 and snap2 ===")
    changes = []
    for key in snap1:
        v1 = snap1[key]
        v2 = snap2.get(key)
        if v1 is None or v2 is None:
            continue
        if v1 != v2:
            changes.append((key, v1, v2))

    if not changes:
        print("  (none)")
        return

    for (name, off), v1, v2 in changes:
        delta = (v2 - v1) & 0xffffffff
        rate = delta / 60.0
        print(f"  {name}+0x{off:05x}  {v1:#010x} -> {v2:#010x}  "
              f"delta={delta:#010x} ({delta:>12d}) rate={rate:>15,.0f}/s")


if __name__ == "__main__":
    main()
