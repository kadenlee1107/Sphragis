#!/usr/bin/env python3
"""Isolated probe: what parts of PCPU cluster MMIO @ 0x211e00000
are actually mapped on M4? Start with the simplest register then
walk outward to see where the valid region ends."""
import sys, pathlib
M1N1 = pathlib.Path(__file__).resolve().parents[2] / "external/m1n1/proxyclient"
sys.path.insert(0, str(M1N1))
from m1n1.setup import *

# Known-good anchors
ECPU = 0x210e00000
PCPU = 0x211e00000

print("=== Probe PCPU @ 0x211e00000 (reads only) ===")
for off in (0x0, 0x40, 0x44, 0x20020, 0x200f8, 0x40000, 0x48000):
    try:
        v = p.read64(PCPU + off)
        print(f"  PCPU +0x{off:05x} @ 0x{PCPU + off:x} = 0x{v:016x}  OK")
    except Exception as e:
        print(f"  PCPU +0x{off:05x} @ 0x{PCPU + off:x} = FAIL: {type(e).__name__}")

print()
print("=== Same offsets on ECPU for comparison ===")
for off in (0x0, 0x40, 0x44, 0x20020, 0x200f8, 0x40000, 0x48000):
    try:
        v = p.read64(ECPU + off)
        print(f"  ECPU +0x{off:05x} = 0x{v:016x}  OK")
    except Exception as e:
        print(f"  ECPU +0x{off:05x} = FAIL: {type(e).__name__}")
