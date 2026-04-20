#!/usr/bin/env python3
"""Try to get PCPU cluster MMIO at 0x211e00000 readable.

First: show what PMGR devices look plausible to poke on. ECPM, PCPM,
PCPU0..3 are all PS-reg devices that stock m1n1 knows about. They
don't have /arm-io paths, so p.pmgr_adt_power_enable() needs a
different hook — try hitting their PS reg directly.

PMGR PS reg format: bits 0-3 = target state, 4-7 = desired,
and bit 8/13 are status. Stock m1n1 sets target=0xf and spins
until desired=0xf. We'll replicate that via raw read32/write32.
"""
import sys, pathlib, time
M1N1 = pathlib.Path("/home/kaden-lee/code/Bat_OS/external/m1n1/proxyclient")
sys.path.insert(0, str(M1N1))
from m1n1.setup import *

PCPU_BASE = 0x211e00000
TEST_OFF = 0x200f8

def pcpu_readable():
    try:
        v = p.read64(PCPU_BASE + TEST_OFF)
        return v, None
    except Exception as e:
        return None, str(e)

# 1. Confirm current state
v, err = pcpu_readable()
print(f"[before] PCPU +0x{TEST_OFF:06x} = {v!r} err={err!r}")

# 2. Dump PS reg state for relevant CPU / cluster devices
print()
print("=== Current PS reg state ===")
candidates = [
    ("ECPU0", 0x380700000),
    ("ECPU1", 0x380700008),
    ("ECPU2", 0x380700010),
    ("ECPU3", 0x380700018),
    ("ECPU4", 0x380700020),
    ("ECPU5", 0x380700028),
    ("PCPU0", 0x380700030),
    ("PCPU1", 0x380700038),
    ("PCPU2", 0x380700040),
    ("PCPU3", 0x380700048),
    ("ECPM",  0x380700050),
    ("PCPM",  0x380700058),
]
for name, addr in candidates:
    try:
        v = p.read32(addr)
        print(f"  {name:8s} @ {addr:#x} = 0x{v:08x}  "
              f"actual={v & 0xf} target={(v>>4) & 0xf} status={(v>>8) & 0xff}")
    except Exception as e:
        print(f"  {name:8s} @ {addr:#x} READ FAILED: {e}")

# 3. Try poking non-boot PCPU cores up to pstate 0xf (target=0xf)
#    If cluster MMIO becomes accessible after this, we've found a
#    PCPU per-core gate. If not, try PCPM (cluster).
print()
print("=== Try wake sequences ===")

def try_wake(name, addr, target_val=0xf):
    try:
        orig = p.read32(addr)
    except Exception as e:
        print(f"  {name} read orig FAILED: {e}")
        return
    # write target bits [4..7] = 0xf
    new = (orig & ~0xf0) | (target_val << 4)
    try:
        p.write32(addr, new)
    except Exception as e:
        print(f"  {name} write FAILED: {e}")
        return
    # Spin until actual[0..3] == target
    for _ in range(1000):
        v = p.read32(addr)
        if (v & 0xf) == target_val:
            break
        time.sleep(0.0005)
    else:
        v = p.read32(addr)
        print(f"  {name} TIMEOUT: {orig:#x} -> {v:#x}")
        return
    # Re-check PCPU MMIO
    readback, err = pcpu_readable()
    print(f"  {name} waked: {orig:#x} -> {v:#x}; "
          f"PCPU +0x{TEST_OFF:06x} = {readback!r} err={err!r}")

# Start with safest: PCPU1 (non-boot P-core)
for (name, addr) in [
    ("PCPU1", 0x380700038),
    ("PCPU2", 0x380700040),
    ("PCPU3", 0x380700048),
]:
    try_wake(name, addr)
    # If it's now readable, stop
    v, err = pcpu_readable()
    if v is not None:
        print(f">>> PCPU MMIO now readable after {name}: v={v:#x}")
        break
