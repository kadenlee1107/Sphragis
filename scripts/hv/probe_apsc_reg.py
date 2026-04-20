#!/usr/bin/env python3
"""Probe the APSC-region register at cluster_base + 0x200f8 on both
ECPU and PCPU. If the reads work, the address is mapped. Write
attempt is what we need to test — but read is safe."""
import sys, pathlib, time
M1N1 = pathlib.Path(__file__).resolve().parents[2] / "external/m1n1/proxyclient"
sys.path.insert(0, str(M1N1))
from m1n1.setup import *

for (name, base) in [("ECPU", 0x210e00000), ("PCPU", 0x211e00000)]:
    for off in (0x200f8, 0x20020, 0x440f8, 0x48400, 0x48408):
        try:
            v = p.read64(base + off)
            print(f"  {name} +0x{off:06x} @ 0x{base + off:x} = 0x{v:016x}")
        except Exception as e:
            print(f"  {name} +0x{off:06x} @ 0x{base + off:x} = READ FAILED: {e}")

# Now try READ-then-WRITE-SAME-VALUE on +0x200f8 to see if the
# write path faults without actually changing any bits.
print()
for (name, base) in [("ECPU", 0x210e00000), ("PCPU", 0x211e00000)]:
    off = 0x200f8
    try:
        v = p.read64(base + off)
        print(f"  {name} read = 0x{v:016x}, writing same value back...")
        p.write64(base + off, v)
        v2 = p.read64(base + off)
        print(f"  {name} readback = 0x{v2:016x}  (noop write survived)")
    except Exception as e:
        print(f"  {name} noop write FAILED: {e}")
