#!/usr/bin/env python3
"""AOP is now at CS=0x4c (ready) from +0x818 probing. Retry aop.start()."""
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
u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

def log(m): print(f"[retry] {m}", flush=True)

AOP = 0x390600000
log(f"pre CC={p.read32(AOP+0x44):#x} CS={p.read32(AOP+0x48):#x} "
    f"+0x40={p.read32(AOP+0x40):#x} +0x818={p.read32(AOP+0x818):#x} "
    f"IB={p.read32(AOP+0x8110):#x} OB={p.read32(AOP+0x8114):#x} "
    f"b14={p.read32(AOP+0xb14):#x}")

from m1n1.hw.dart import DART
from m1n1.fw.aop.client import AOPClient

dart_node = u.adt["/arm-io/dart-aop"]
vm_base = getattr(dart_node, "vm-base", None) or 0x8000
dart = DART.from_adt(u, "/arm-io/dart-aop",
                     iova_range=(vm_base, 0x1000000000))
dart.initialize()

aop = AOPClient(u, "/arm-io/aop", dart)
aop.verbose = 3

# Do NOT clear RUN — AOP is in a specific state we don't want to disturb
log("calling aop.mgmt.start() directly (skip boot())")
try:
    aop.mgmt.start()  # sends SetIOPPower(0x220)
    log("sent SetIOPPower, polling for reply...")
    deadline = time.time() + 3
    while time.time() < deadline:
        aop.work()
        if aop.mgmt.iop_power_state == 0x20 and aop.mgmt.ap_power_state == 0x20:
            log("*** BOOT COMPLETE ***")
            break
        time.sleep(0.05)
    else:
        log("still waiting... iop_power=%x ap_power=%x" %
            (aop.mgmt.iop_power_state, aop.mgmt.ap_power_state))
except Exception as e:
    log(f"FAIL: {type(e).__name__}: {e}")

log(f"post CC={p.read32(AOP+0x44):#x} CS={p.read32(AOP+0x48):#x} "
    f"+0x40={p.read32(AOP+0x40):#x} +0x818={p.read32(AOP+0x818):#x} "
    f"IB={p.read32(AOP+0x8110):#x} OB={p.read32(AOP+0x8114):#x} "
    f"b14={p.read32(AOP+0xb14):#x}")

# Peek OUTBOX for any msg
if not (p.read32(AOP + 0x8114) & (1 << 17)):
    msg0 = p.read32(AOP + 0x8830) | (p.read32(AOP + 0x8834) << 32)
    msg1 = p.read32(AOP + 0x8838) | (p.read32(AOP + 0x883c) << 32)
    log(f"OUTBOX msg0={msg0:#x} msg1={msg1:#x}")

os._exit(0)
