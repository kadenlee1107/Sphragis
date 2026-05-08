#!/usr/bin/env python3
"""Scan AOP's ASC reg[0] (0x390600000..0x390688000) looking for the real
'CS bit 2 set' transition path, and diff against MTP which works.

Strategy:
  1. Snap all non-zero u32s in first 0x1000 of AOP reg[0]
  2. Snap same for MTP reg[0] (since MTP advances past first msg)
  3. Report non-matching offsets
"""
import os
import pathlib
import sys
ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

iface = UartInterface()
p = M1N1Proxy(iface, debug=False)
bootstrap_port(iface, p)
u = ProxyUtils(p, heap_size=8 * 1024 * 1024)

AOP = 0x390600000
MTP = 0x394600000

def dump(base, name, stop=0x400, step=4):
    print(f"=== {name} @ {base:#x} (0..{stop:#x}) ===", flush=True)
    for off in range(0, stop, step):
        v = p.read32(base + off)
        if v != 0:
            print(f"  +{off:#06x} = {v:#010x}")

def dump_range(base, name, start, stop, step=4):
    print(f"=== {name} @ {base:#x} range {start:#x}..{stop:#x} ===", flush=True)
    for off in range(start, stop, step):
        v = p.read32(base + off)
        if v != 0:
            print(f"  +{off:#06x} = {v:#010x}")

# 0..0x100: CPU control block (CC @0x44, CS @0x48 known)
for base, nm in [(AOP, "AOP"), (MTP, "MTP")]:
    dump(base, nm + " [0..0x200]", 0x200, 4)

# 0x400..0x500: saw +0x400 = 0x400 post-boot on MTP
for base, nm in [(AOP, "AOP"), (MTP, "MTP")]:
    dump_range(base, nm + " [0x400..0x500]", 0x400, 0x500, 4)

# 0x800..0x900
for base, nm in [(AOP, "AOP"), (MTP, "MTP")]:
    dump_range(base, nm + " [0x800..0x900]", 0x800, 0x900, 4)

# 0xa00..0xb00
for base, nm in [(AOP, "AOP"), (MTP, "MTP")]:
    dump_range(base, nm + " [0xa00..0xb00]", 0xa00, 0xb00, 4)

# 0xb00..0xc00 (+b14 region)
for base, nm in [(AOP, "AOP"), (MTP, "MTP")]:
    dump_range(base, nm + " [0xb00..0xc00]", 0xb00, 0xc00, 4)

# 0xc00..0xd00
for base, nm in [(AOP, "AOP"), (MTP, "MTP")]:
    dump_range(base, nm + " [0xc00..0xd00]", 0xc00, 0xd00, 4)

# 0x4000..0x4800 — attestation-like slots seen at +0x4400..+0x4700
# (4 rows, 3 random u32 + 0x00020000 trailer on AOP).
# Check if MTP has equivalent table — that tells us if it's AOP-specific.
for base, nm in [(AOP, "AOP"), (MTP, "MTP")]:
    dump_range(base, nm + " [0x4000..0x4800]", 0x4000, 0x4800, 4)

# 0x8000..0x8200 — +0x8000 had CS-like 0x4c on AOP. DO NOT read past
# +0x8200 — DAPF SYNC wedges m1n1.
for base, nm in [(AOP, "AOP"), (MTP, "MTP")]:
    dump_range(base, nm + " [0x8000..0x8200]", 0x8000, 0x8200, 4)

os._exit(0)
