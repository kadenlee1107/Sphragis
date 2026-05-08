#!/usr/bin/env python3
"""AOP FW woke and acked our INBOX via +0x818 bit 3, but standard OUTBOX
stays empty. Scan broader for where FW might have written a reply."""
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

def log(m): print(m, flush=True)

# Scan 0..0x2000 at 4B granularity (below the mailbox region we know)
log("=== 0..0x1000 (CPU ctrl) ===")
for off in range(0, 0x1000, 4):
    v = p.read32(AOP + off)
    if v != 0:
        log(f"  +{off:#06x} = {v:#010x}")

log("\n=== 0x1000..0x2000 ===")
for off in range(0x1000, 0x2000, 4):
    v = p.read32(AOP + off)
    if v != 0:
        log(f"  +{off:#06x} = {v:#010x}")

log("\n=== 0x2000..0x3000 ===")
for off in range(0x2000, 0x3000, 4):
    v = p.read32(AOP + off)
    if v != 0:
        log(f"  +{off:#06x} = {v:#010x}")

log("\n=== 0x7000..0x8200 (leading up to mailbox, but not past 0x8200) ===")
for off in range(0x7000, 0x8200, 4):
    v = p.read32(AOP + off)
    if v != 0:
        log(f"  +{off:#06x} = {v:#010x}")

# Already know 0x8000..0x8200 from earlier. Check +0x8800 mailbox region
log("\n=== 0x8800..0x8900 (mailbox area) ===")
for off in range(0x8800, 0x8900, 4):
    v = p.read32(AOP + off)
    if v != 0:
        log(f"  +{off:#06x} = {v:#010x}")

os._exit(0)
