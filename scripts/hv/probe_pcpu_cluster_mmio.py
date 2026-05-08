#!/usr/bin/env python3
"""SError-safe probe of P-cluster MMIO.

Strategy:
  1. First dump PS regs (safe, PMGR status at 0x380700xxx).
  2. Then sweep PCPU cluster MMIO for accessible offsets — but use
     p.read64 with an exception handler per-offset. If the proxy
     survives (m1n1 SError handler returns instead of halting on M4),
     we'll build a map. If the FIRST SError wedges the proxy, we
     stop right there and move to Plan B.
"""
import sys
sys.path.insert(0, "/home/kaden-lee/code/Bat_OS/external/m1n1/proxyclient")
from m1n1.setup import *

# 1) PS regs first (can't SError — they're at 0x380700xxx, not cluster MMIO)
print("=== PS regs (PMGR status) ===")
for (name, addr) in [
    ("ECPU0", 0x380700000),("ECPU1",0x380700008),("ECPU2",0x380700010),
    ("ECPU3", 0x380700018),("ECPU4",0x380700020),("ECPU5",0x380700028),
    ("PCPU0", 0x380700030),("PCPU1",0x380700038),("PCPU2",0x380700040),
    ("PCPU3", 0x380700048),("ECPM", 0x380700050),("PCPM", 0x380700058),
]:
    try:
        v = p.read32(addr)
        t = v & 0xf; a = (v >> 4) & 0xf
        print(f"  {name:6s} @ {addr:#x} = {v:#010x}  target={t:#x} actual={a:#x}")
    except Exception as e:
        print(f"  {name:6s} @ {addr:#x} FAILED: {e}")
        sys.exit(1)

# 2) Now sweep cluster MMIO at known-interesting offsets. If the
#    first SError wedges the proxy, remaining probes will fail —
#    caller should power-cycle and try fewer.
print()
print("=== P-cluster MMIO read sweep ===")
PCPU_BASE = 0x211e00000
safe_offsets = [
    0x0, 0x8, 0x10, 0x1000, 0x20000, 0x20010, 0x20020, 0x20030,
    0x20040, 0x20080, 0x200f0, 0x200f8, 0x20100, 0x40000, 0x48000,
    0x48400, 0x48408, 0x80000, 0x100000,
]
for off in safe_offsets:
    addr = PCPU_BASE + off
    try:
        v = p.read64(addr)
        print(f"  PCPU +0x{off:06x} @ {addr:#x} = {v:#018x}")
    except Exception as e:
        print(f"  PCPU +0x{off:06x} @ {addr:#x} FAILED ({type(e).__name__}): {e}")
        # If proxy is dead, exit
        try:
            p.iodev_whoami()
        except Exception:
            print("!!! proxy is dead after SError. Stop.")
            sys.exit(2)

print()
print("=== E-cluster MMIO read sweep (for comparison) ===")
ECPU_BASE = 0x210e00000
for off in safe_offsets:
    addr = ECPU_BASE + off
    try:
        v = p.read64(addr)
        print(f"  ECPU +0x{off:06x} @ {addr:#x} = {v:#018x}")
    except Exception as e:
        print(f"  ECPU +0x{off:06x} @ {addr:#x} FAILED ({type(e).__name__}): {e}")
        try:
            p.iodev_whoami()
        except Exception:
            print("!!! proxy is dead after SError. Stop.")
            sys.exit(2)
