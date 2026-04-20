#!/usr/bin/env python3
"""Walk the M4 ADT via the stock-m1n1 proxy and dump every node whose
name/compatible hints at a watchdog or keepalive timer we haven't
disabled. Also read the register state of the known WDT block at
0x3882b0000 so we can see whether wdt_disable() actually landed.

Usage:
  sg dialout -c "M1N1DEVICE=/dev/ttyACM1 \
      /usr/bin/python3 scripts/hv/probe_m4_watchdogs.py"

Must be run against stock (un-chainloaded) m1n1 — patched m1n1
under HV owns the proxy and this script won't attach.
"""
import os, sys, pathlib
M1N1 = pathlib.Path(__file__).resolve().parents[2] / "external/m1n1/proxyclient"
sys.path.insert(0, str(M1N1))
from m1n1.setup import *  # brings in p, u, iface, u.adt

HINTS = ("wdt", "watchdog", "keepalive", "heartbeat", "timeout",
         "alarm", "aop", "ans", "sep-wdt")


def walk(node, path):
    try:
        name = node.name
    except Exception:
        name = "<?>"
    # ADT 'compatible' is a str or tuple depending on pyadt version.
    compat = ""
    try:
        compat = str(node.compatible)
    except Exception:
        pass

    hit = [h for h in HINTS if h in name.lower() or h in compat.lower()]
    if hit:
        print(f"  {path}/{name}: compat={compat} hints={hit}")
        try:
            regs = node.get_reg(0)
            print(f"      reg[0] = {regs!r}")
        except Exception as e:
            print(f"      (no reg: {e})")

    try:
        children = list(node)
    except Exception:
        return
    for c in children:
        walk(c, f"{path}/{name}")


print("=== M4 ADT watchdog-ish node scan ===")
walk(u.adt, "")

print("\n=== Known WDT block dump (0x3882b0000, first 0x40) ===")
for off in range(0, 0x40, 4):
    print(f"  0x{off:03x}: 0x{p.read32(0x3882b0000 + off):08x}")

print("\n=== /arm-io/wdt reg from ADT ===")
try:
    n = u.adt["arm-io/wdt"]
    print(f"  node: {n.name}, compat={n.compatible}, reg={n.get_reg(0)}")
except Exception as e:
    print(f"  lookup failed: {e}")
