#!/usr/bin/env python3
"""AOP is alive from boot_aop_no_dart.py (CS=0x68). Try:
1. Targeted dapf_init for /arm-io/dart-aop (not dart-mtp which hangs).
2. Send SetIOPPower.
3. Poll OUTBOX thoroughly.
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
u = ProxyUtils(p, heap_size=32 * 1024 * 1024)

AOP = 0x390600000
def log(m): print(f"[dapf-start] {m}", flush=True)

def snap(tag):
    cc = p.read32(AOP + 0x44)
    cs = p.read32(AOP + 0x48)
    ib = p.read32(AOP + 0x8110)
    ob = p.read32(AOP + 0x8114)
    ob0 = p.read32(AOP + 0x8830) | (p.read32(AOP + 0x8834) << 32)
    ob1 = p.read32(AOP + 0x8838) | (p.read32(AOP + 0x883c) << 32)
    r818 = p.read32(AOP + 0x818)
    log(f"[{tag}] CC={cc:#x} CS={cs:#x} IB={ib:#x} OB={ob:#x} +0x818={r818:#x} OUT0={ob0:#x} OUT1={ob1:#x}")

snap("initial")

log("calling p.dapf_init('/arm-io/dart-aop') with 10s timeout...")
saved = iface.dev.timeout
iface.dev.timeout = 10
t0 = time.time()
try:
    rc = p.dapf_init("/arm-io/dart-aop")
    log(f"  dapf_init rc={rc} in {(time.time()-t0)*1000:.0f}ms")
except Exception as e:
    log(f"  dapf_init FAIL after {(time.time()-t0)*1000:.0f}ms: {type(e).__name__}: {e}")
iface.dev.timeout = saved

snap("post-dapf")

# Now send SetIOPPower
log("sending SetIOPPower(0x220)...")
msg0 = 0x60000000000220  # TYPE=6, STATE=0x220
p.write32(AOP + 0x8800, msg0 & 0xffffffff)
p.write32(AOP + 0x8804, (msg0 >> 32) & 0xffffffff)
p.write32(AOP + 0x8808, 0)  # msg1 low (EP=0)
p.write32(AOP + 0x880c, 0)  # msg1 high

snap("post-send")

# Poll hard for OUTBOX
log("polling 5s for OUTBOX...")
deadline = time.time() + 5
last_ob = None
got = False
while time.time() < deadline:
    ob = p.read32(AOP + 0x8114)
    if ob != last_ob:
        log(f"  t={time.time() - deadline + 5:.2f}s OB={ob:#x}")
        last_ob = ob
    if not (ob & (1 << 17)):  # not EMPTY
        ob0 = p.read32(AOP + 0x8830) | (p.read32(AOP + 0x8834) << 32)
        ob1 = p.read32(AOP + 0x8838) | (p.read32(AOP + 0x883c) << 32)
        typ = (ob0 >> 52) & 0xff
        log(f"  *** OUTBOX msg! msg0={ob0:#x} msg1={ob1:#x} TYPE={typ:#x} ***")
        got = True
        break
    time.sleep(0.05)

snap("final")

if got:
    log("*** AOP OUTBOX RESPONSIVE ***")
else:
    log("still no OUTBOX")

os._exit(0)
