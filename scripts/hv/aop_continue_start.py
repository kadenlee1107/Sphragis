#!/usr/bin/env python3
"""AOP is alive at CS=0x6c. Try proper AOPClient flow with full polling.
Skip dart.initialize() to preserve iBoot DART config."""
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
def log(m): print(f"[cont] {m}", flush=True)

# Construct DART object from ADT but DO NOT call initialize()
from m1n1.hw.dart import DART
dart_node = u.adt["/arm-io/dart-aop"]
vm_base = getattr(dart_node, "vm-base", None) or 0x8000
dart = DART.from_adt(u, "/arm-io/dart-aop", iova_range=(vm_base, 0x1000000000))
# NOTE: skipping dart.initialize() — leave iBoot config intact
log("DART constructed (no init)")

from m1n1.fw.aop.client import AOPClient
aop = AOPClient(u, "/arm-io/aop", dart)
aop.verbose = 3

# Poll mgmt work via asc.work() to process any pending OUTBOX
log("calling asc.work_for(2.0) to drain any pending OUTBOX...")
aop.work_for(2.0)

cs = p.read32(AOP + 0x48)
log(f"CS after work_for = {cs:#x}")

# Try start endpoints even without boot-complete
log("attempting start_ep for each endpoint...")
for epno in [0x20, 0x21, 0x22, 0x24, 0x25, 0x26, 0x27, 0x28]:
    try:
        aop.start_ep(epno)
        log(f"  ep {epno:#x} start_ep sent")
    except Exception as e:
        log(f"  ep {epno:#x}: {type(e).__name__}: {e}")
    aop.work_for(0.5)
    ob = p.read32(AOP + 0x8114)
    log(f"    OB={ob:#x}")

log("final 3s work_for to see any late reply...")
aop.work_for(3.0)

cs = p.read32(AOP + 0x48)
ib = p.read32(AOP + 0x8110)
ob = p.read32(AOP + 0x8114)
log(f"final: CS={cs:#x} IB={ib:#x} OB={ob:#x} "
    f"iop_power={aop.mgmt.iop_power_state:#x} "
    f"ap_power={aop.mgmt.ap_power_state:#x}")

os._exit(0)
