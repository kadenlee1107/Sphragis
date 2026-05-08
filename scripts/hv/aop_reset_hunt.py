#!/usr/bin/env python3
"""Hunt for ascwrap-v6 CPU reset register on M4.

AOP is stuck with CS=0x48 (bit 2 clear = IRQ pending / pre-boot stuck).
Writing RUN=0 to CC doesn't help — CS stays at 0x48.

Probe: snapshot registers in likely reset candidate ranges, try writing
patterns, watch CS for any transition.

Candidate offsets to try (based on M1/M2 ASC + intuition for v6):
  +0x0040  CPU_unk0   0x000a0000 (pre-boot stage marker?)
  +0x0044  CPU_CONTROL (RUN)
  +0x0048  CPU_STATUS (observe)
  +0x004c
  +0x0050
  +0x0100..0x0200  possible per-CPU regs
  +0x0400 / +0x0404 / +0x0444 (alias?)
  +0x0800..0x081c (boot marker region)
  +0x0b10 / 0xb14 (we know +b14 is progress)
"""
import os
import pathlib
import sys
import time

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

def cs(): return p.read32(AOP + 0x48)
def cc(): return p.read32(AOP + 0x44)
def b14(): return p.read32(AOP + 0xb14)
def snap(tag):
    print(f"[{tag}] CC={cc():#x} CS={cs():#x} b14={b14():#x}", flush=True)

snap("start")

# Attempt 1: toggle bit 4 of various alias CC regs
print("\n--- Attempt 1: write 0 to CPU_unk0 (+0x40) ---", flush=True)
old = p.read32(AOP + 0x40); print(f"  old +0x40 = {old:#x}")
p.write32(AOP + 0x40, 0)
time.sleep(0.1)
now = p.read32(AOP + 0x40); print(f"  after w=0 +0x40 = {now:#x}")
snap("post-40=0")
p.write32(AOP + 0x40, old); print(f"  restored to {old:#x}")

# Attempt 2: write to +0x4c, +0x50, +0x54
print("\n--- Attempt 2: poke +0x4c/50/54 to see which is volatile ---", flush=True)
for off in (0x4c, 0x50, 0x54, 0x58, 0x5c, 0x60):
    pre = p.read32(AOP + off)
    print(f"  +{off:#x} = {pre:#x}")

# Attempt 3: try writing specific reset magic patterns to +0x44 high bits
print("\n--- Attempt 3: write HALT/RESET bits to CC ---", flush=True)
for val in (0x20, 0x40, 0x80, 0x100, 0x200, 0x400, 0x800, 0x1000, 0x10000, 0x100000, 0x1000000, 0x10000000, 0x80000000):
    p.write32(AOP + 0x44, val)
    time.sleep(0.05)
    new_cc = cc(); new_cs = cs()
    print(f"  w CC={val:#x} → CC={new_cc:#x} CS={new_cs:#x}")
    if new_cs != 0x48 and new_cs != 0x58:
        print(f"    *** CS CHANGED from 0x48 to {new_cs:#x} ***")

# Restore CC
p.write32(AOP + 0x44, 0x10)
snap("post-cc-hunt")

# Attempt 4: try writing to +0x400, +0x404, +0x408
print("\n--- Attempt 4: probe 0x400 region ---", flush=True)
for off in (0x400, 0x404, 0x408, 0x40c, 0x410, 0x414, 0x440, 0x444, 0x448, 0x44c):
    pre = p.read32(AOP + off)
    print(f"  +{off:#x} = {pre:#x}")

# Attempt 5: try reset by writing to boot config +0x818
print("\n--- Attempt 5: toggle +0x818 (FW consumed low bits) ---", flush=True)
pre = p.read32(AOP + 0x818)
print(f"  pre +0x818 = {pre:#x}")
p.write32(AOP + 0x818, 0x00040003)  # restore iBoot value
time.sleep(0.05)
now = p.read32(AOP + 0x818)
print(f"  after restore = {now:#x}")
snap("post-818=3")

# Attempt 6: try writing to unseen +0xc00 and +0xd00 range, walking strides
print("\n--- Attempt 6: walk +0xb00 range ---", flush=True)
for off in range(0xb00, 0xc00, 4):
    v = p.read32(AOP + off)
    if v != 0:
        print(f"  +{off:#x} = {v:#x}")

# Attempt 7: try +0x40fc / +0x4100 / +0x40f0 (SRAM-adjacent)
print("\n--- Attempt 7: scan 0x4000 region ---", flush=True)
for off in range(0x4000, 0x4100, 4):
    v = p.read32(AOP + off)
    if v != 0:
        print(f"  +{off:#x} = {v:#x}")

snap("final")
os._exit(0)
