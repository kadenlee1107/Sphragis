#!/usr/bin/env python3
"""Probe DART-AOP state after our boot_aop.py to see if our
dart.initialize() wiped iBoot's mappings."""
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

def log(m): print(m, flush=True)

# DART-AOP regs from ADT: reg[0]=0x390f00000 size=0xc000 (main)
#                         reg[1]=0x390f10000 size=0x4000 (TCR?)
#                         reg[2]=0x3882a4000 size=0x4000 (clocks/power)
DA0 = 0x390f00000
DA1 = 0x390f10000

log("DART-AOP reg[0] scan (first 0x400 bytes):")
for off in range(0, 0x400, 4):
    try:
        v = p.read32(DA0 + off)
        if v != 0:
            log(f"  reg[0]+{off:#x} = {v:#x}")
    except Exception as e:
        log(f"  reg[0]+{off:#x} ERR: {e}")
        break

log("\nDART-AOP reg[1] scan:")
for off in range(0, 0x100, 4):
    try:
        v = p.read32(DA1 + off)
        if v != 0:
            log(f"  reg[1]+{off:#x} = {v:#x}")
    except Exception as e:
        log(f"  reg[1]+{off:#x} ERR: {e}")
        break

# DART-MTP for comparison
log("\nDART-MTP reg[0] scan:")
DM0 = 0x394800000
for off in range(0, 0x400, 4):
    try:
        v = p.read32(DM0 + off)
        if v != 0:
            log(f"  dart-mtp reg[0]+{off:#x} = {v:#x}")
    except:
        break

os._exit(0)
