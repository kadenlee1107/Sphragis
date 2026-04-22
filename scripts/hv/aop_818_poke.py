#!/usr/bin/env python3
"""Probe AOP's +0x818 handshake register.

Evidence so far:
  iBoot pre: 0x40003 (FW hasn't booted yet)
  post-run-attempt: 0x40000 (FW cleared low bits)
  after we wrote 0x40003 back: read as 0x40005 (FW cleared bit 1, set bit 2)

So FW actively state-machines through low bits. Document the transitions.
"""
import os, pathlib, sys, time

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

def snap(tag):
    cc  = p.read32(AOP + 0x44)
    cs  = p.read32(AOP + 0x48)
    u40 = p.read32(AOP + 0x40)
    r818 = p.read32(AOP + 0x818)
    ib = p.read32(AOP + 0x8110)
    ob = p.read32(AOP + 0x8114)
    b14 = p.read32(AOP + 0xb14)
    print(f"[{tag}] CC={cc:#x} CS={cs:#x} +0x40={u40:#x} +0x818={r818:#x} IB={ib:#x} OB={ob:#x} b14={b14:#x}", flush=True)


snap("initial")

# Walk every bit of +0x818 and see the FW response
print("\n--- Write bits 0..7 one at a time to +0x818 and observe ---", flush=True)
for bit in range(8):
    val = 0x40000 | (1 << bit)
    p.write32(AOP + 0x818, val)
    time.sleep(0.1)
    got = p.read32(AOP + 0x818)
    b14v = p.read32(AOP + 0xb14)
    u40v = p.read32(AOP + 0x40)
    print(f"  wrote {val:#x} → read {got:#x}  +0x40={u40v:#x} b14={b14v:#x}")

# Walk wider
print("\n--- Try 0x40001, 0x40002, 0x40003 sequence (Hello handshake?) ---", flush=True)
for val in (0x40001, 0x40002, 0x40003, 0x40004, 0x40005, 0x40006, 0x40007, 0x40008, 0x4000f):
    p.write32(AOP + 0x818, val)
    time.sleep(0.15)
    got = p.read32(AOP + 0x818)
    b14v = p.read32(AOP + 0xb14)
    u40v = p.read32(AOP + 0x40)
    cs_v = p.read32(AOP + 0x48)
    ob_v = p.read32(AOP + 0x8114)
    print(f"  wrote {val:#x} → read {got:#x}  CS={cs_v:#x} +0x40={u40v:#x} OB={ob_v:#x} b14={b14v:#x}")

# After sequence, check OUTBOX for any msg from FW
print("\n--- Check OUTBOX for FW Hello ---", flush=True)
ob_ctrl = p.read32(AOP + 0x8114)
print(f"OUTBOX_CTRL = {ob_ctrl:#x}")
if not (ob_ctrl & (1 << 17)):  # not EMPTY
    out0 = p.read32(AOP + 0x8830); out1 = p.read32(AOP + 0x8838)
    print(f"  OUTBOX0 = {out0:#x}, OUTBOX1 = {out1:#x}")
else:
    print("  OUTBOX empty")

snap("final")

os._exit(0)
