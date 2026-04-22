#!/usr/bin/env python3
"""Wide scan: look for non-zero/interesting regs across AOP reg[0] 0..0x80000
to see if ascwrap-v6 has alt mailbox or boot regs at nonstandard offsets."""
import os, pathlib, sys
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
SIZE = 0x80000  # aop reg[0] is 0x88000, keep safe

# Strategy: read every 4B at 0x200 stride to find non-zero regions quickly
print(f"Scanning {AOP:#x}..{AOP+SIZE:#x} at 0x100 stride, 16B per hit...", flush=True)

nonzero = []
for off in range(0, SIZE, 0x100):
    v = p.read32(AOP + off)
    if v != 0:
        nonzero.append(off)
        w2 = p.read32(AOP + off + 4)
        w3 = p.read32(AOP + off + 8)
        w4 = p.read32(AOP + off + 12)
        print(f"  +{off:#08x} = {v:#010x} {w2:#010x} {w3:#010x} {w4:#010x}", flush=True)

print(f"\nFound {len(nonzero)} nonzero regions of size >= 0x100", flush=True)

os._exit(0)
