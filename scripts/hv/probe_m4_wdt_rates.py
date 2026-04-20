#!/usr/bin/env python3
"""Probe the M4 WDT block at 0x3882b0000 — read every register pair
twice with a known wall-clock delay so we can see:
  (a) which counters are actively running,
  (b) their tick rate (→ identify the clock source),
  (c) what CTL bits enable each instance.

Run against stock m1n1 only.
"""
import sys, pathlib, time
M1N1 = pathlib.Path(__file__).resolve().parents[2] / "external/m1n1/proxyclient"
sys.path.insert(0, str(M1N1))
from m1n1.setup import *

WDT = 0x3882b0000

def snap(label):
    print(f"--- {label} ---")
    vals = []
    for off in range(0, 0x40, 4):
        vals.append(p.read32(WDT + off))
        print(f"  0x{off:03x}: 0x{vals[-1]:08x}")
    return vals

a = snap("t0")
t0 = time.time()
time.sleep(1.5)
b = snap("t1 (+1.5s)")
t1 = time.time()

print(f"\ndelta ({t1 - t0:.3f}s wall):")
for off in range(0, 0x40, 4):
    d = (b[off // 4] - a[off // 4]) & 0xffffffff
    if d:
        rate = d / (t1 - t0)
        print(f"  0x{off:03x}: +0x{d:08x}  ({d} = ~{rate:,.0f}/s)")
