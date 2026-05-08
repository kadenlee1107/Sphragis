#!/usr/bin/env python3
"""Hunt for the MMIO address that ticks down on the M4's ~118 s
wall-clock watchdog.

Approach:
  1. Use the already-chainloaded patched m1n1 (supervisor has done it).
  2. Call hv.init() via proxy — per this session's findings, hv.init
     arms the 118 s timer.
  3. Read a snapshot of every suspect MMIO once per 5 s for 130 s.
  4. After the run, diff across timepoints and print only the
     addresses whose values CHANGED — that's the ticking counter.

We expect ACM2 (vuart) to drop around t=115 s (firing is partial
when hv.start is never called — see 2026-04-20 19:15 journal entry).
ACM1 (proxy) should stay up for the full 130 s.

Run while the Mac is in stock m1n1 waiting:
  sg dialout -c "/usr/bin/python3 scripts/hv/probe_118s_timer_hunt.py"

If the Mac is in a patched m1n1 still running from a prior session,
re-chainload first:
  sudo -n M1N1DEVICE=/dev/ttyACM1 M1N1WAIT=1 \
       /usr/bin/python3 external/m1n1/proxyclient/tools/chainload.py \
       -S external/m1n1/build/m1n1.macho
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
from m1n1.hv import HV


def _safe_read32(p, addr):
    """Try a read; if it SErrors the proxy, return None and caller
    should check proxy liveness via iodev_whoami()."""
    try:
        return p.read32(addr)
    except Exception:
        return None


def probe(p, u):
    """One snapshot of every suspect MMIO.
    EL2-safe reads only. Grown incrementally; any address added
    here must have survived a prior 130 s probe run without SErr."""
    snap = {}

    # --- PMGR PS regs — known safe, per dump_pmgr (survived 130 s) ---
    pmgr_base = 0x380700000
    for off in (0x0, 0x8, 0x30, 0x48, 0x50, 0x58):
        snap[f"pmgr+0x{off:03x}"] = _safe_read32(p, pmgr_base + off)

    # --- SoC WDT at 0x3882b0000 — m1n1 disabled it, so CTL=0.
    # Per Apple's WDT layout (src/wdt.c): COUNT=0x10 ALARM=0x14 CTL=0x1c ---
    for off in (0x10, 0x14, 0x1c,           # first instance
                0x30, 0x34, 0x3c,            # second (chip-WDT — already 0)
                0x50, 0x54, 0x5c):           # third
        snap[f"wdt+0x{off:02x}"] = _safe_read32(p, 0x3882b0000 + off)

    # --- Dockchannel UART at 0x388128000 — known safe, guest traps
    # these every MMIO poll. Only reading TX_FREE / RX_COUNT. ---
    for off in (0x4014, 0x402c):
        snap[f"dc+0x{off:04x}"] = _safe_read32(p, 0x388128000 + off)

    snap["_host_mono_ms"] = int(time.monotonic() * 1000)

    return snap


def main():
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    print("[hunt] proxy alive", flush=True)

    # Call hv.init() to ARM the 118 s timer (per today's findings).
    # Skip with HUNT_SKIP_HV_INIT=1 to do the hunt in stock m1n1 for
    # baseline comparison.
    if os.environ.get("HUNT_SKIP_HV_INIT", "0") == "1":
        print("[hunt] HUNT_SKIP_HV_INIT=1 — not arming the watchdog, "
              "baseline only", flush=True)
    else:
        print("[hunt] calling hv.init() to arm the 118 s watchdog…", flush=True)
        hv = HV(iface, p, u)
        hv.init()
        print("[hunt] hv.init() done, starting timer hunt", flush=True)

    # Snapshot loop — every 5 s for 130 s
    t0 = time.monotonic()
    snapshots = []
    while True:
        elapsed = time.monotonic() - t0
        if elapsed > 135:
            break
        print(f"[hunt] t={elapsed:6.1f}s snapshotting…", flush=True)
        try:
            snap = probe(p, u)
            snap["_elapsed_s"] = int(elapsed)
            snapshots.append(snap)
        except Exception as e:
            print(f"[hunt] snapshot failed at t={elapsed:.1f}s: {e}", flush=True)
            break
        # Sleep the remainder of the 5 s window
        next_target = (len(snapshots)) * 5.0
        while time.monotonic() - t0 < next_target:
            time.sleep(0.1)

    # Find counters that CHANGED across the run
    print()
    print(f"=== changed MMIO across {len(snapshots)} snapshots ===")
    if not snapshots:
        print("NO SNAPSHOTS — proxy died before first probe")
        return
    first = snapshots[0]
    for key in sorted(first.keys()):
        if key.startswith("_") or key.endswith("_err"):
            continue
        vals = [s.get(key) for s in snapshots]
        if any(v != vals[0] for v in vals if v is not None):
            print(f"  {key:25s}  " + "  ".join(
                f"t={s['_elapsed_s']}s:{v:#010x}" if isinstance(v, int) else f"t={s['_elapsed_s']}s:ERR"
                for s, v in zip(snapshots, vals)))
    print()
    print("=== constant addresses (hidden) ===")
    constant = 0
    for key in sorted(first.keys()):
        if key.startswith("_") or key.endswith("_err"):
            continue
        vals = [s.get(key) for s in snapshots]
        if all(v == vals[0] for v in vals if v is not None):
            constant += 1
    print(f"  {constant} addresses did not change across the run")


if __name__ == "__main__":
    main()
