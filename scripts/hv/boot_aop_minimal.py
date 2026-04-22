#!/usr/bin/env python3
"""Minimal AOP boot: NO staging, NO bootargs changes. Just kick RUN.

Theory: iBoot may have pre-staged __DATA + bootargs too, not just __TEXT.
Our script's overwrites may corrupt state. Minimal test: just flip RUN=1
and observe.

Run order: this is EXPERIMENT 1. Run boot_aop.py baseline separately
(it fails) — this minimal tests if OUR staging is the cause.
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
u = ProxyUtils(p, heap_size=32 * 1024 * 1024)

AOP = 0x390600000
def log(m): print(f"[min] {m}", flush=True)

def snap(tag):
    cc = p.read32(AOP + 0x44)
    cs = p.read32(AOP + 0x48)
    ib = p.read32(AOP + 0x8110)
    ob = p.read32(AOP + 0x8114)
    b14 = p.read32(AOP + 0xb14)
    u40 = p.read32(AOP + 0x40)
    log(f"[{tag}] CC={cc:#x} CS={cs:#x} IB={ib:#x} OB={ob:#x} b14={b14:#x} +0x40={u40:#x}")

# Disable watchdog first
for addr, val in [(0x3882BC224, 0), (0x3882B8008, 0xffffffff),
                  (0x3882B802C, 0xffffffff), (0x3882B8020, 0xffffffff)]:
    try: p.write32(addr, val)
    except: pass

snap("pre-RUN")

# Just kick RUN=1
log("writing CC.RUN=1...")
p.write32(AOP + 0x44, 0x10)
time.sleep(0.5)

snap("post-RUN")

# Wait 3s and watch OUTBOX — FW should send Hello
log("watching OUTBOX for 3s...")
deadline = time.time() + 3
last_cs = None
while time.time() < deadline:
    cs = p.read32(AOP + 0x48)
    ob = p.read32(AOP + 0x8114)
    u40 = p.read32(AOP + 0x40)
    if cs != last_cs:
        log(f"  t={time.time()-deadline+3:.2f}s CS={cs:#x} OB={ob:#x} +0x40={u40:#x}")
        last_cs = cs
    if not (ob & (1 << 17)):
        ob0 = p.read32(AOP + 0x8830) | (p.read32(AOP + 0x8834) << 32)
        ob1 = p.read32(AOP + 0x8838) | (p.read32(AOP + 0x883c) << 32)
        log(f"  *** OUTBOX msg0={ob0:#x} msg1={ob1:#x} ***")
        # Consume it
        ob_after = p.read32(AOP + 0x8114)
        log(f"  after read, OB_CTRL={ob_after:#x}")
        break
    time.sleep(0.05)
else:
    log("  no OUTBOX activity")

snap("final")
os._exit(0)
