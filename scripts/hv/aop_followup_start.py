#!/usr/bin/env python3
"""AOP is now idle (CS=0x68) after boot_aop_no_dart.py. Send SetIOPPower
and see if FW responds this time."""
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
def log(m): print(f"[fu] {m}", flush=True)

def snap(tag):
    cc = p.read32(AOP + 0x44)
    cs = p.read32(AOP + 0x48)
    ib = p.read32(AOP + 0x8110)
    ob = p.read32(AOP + 0x8114)
    b14 = p.read32(AOP + 0xb14)
    u40 = p.read32(AOP + 0x40)
    r818 = p.read32(AOP + 0x818)
    log(f"[{tag}] CC={cc:#x} CS={cs:#x} IB={ib:#x} OB={ob:#x} b14={b14:#x} "
        f"+0x40={u40:#x} +0x818={r818:#x}")

snap("initial")

# Send SetIOPPower(STATE=0x220) — the same msg mgmt.start sends.
# TYPE=0x6, STATE=0x220 → msg0 = (0x6 << 52) | 0x220 = 0x60000000000220
# msg1 = EP 0 (mgmt endpoint)
log("sending SetIOPPower(0x220)...")
msg0 = 0x60000000000220
msg1 = 0x0  # EP=0 (mgmt)
p.write32(AOP + 0x8800, msg0 & 0xffffffff)
p.write32(AOP + 0x8804, (msg0 >> 32) & 0xffffffff)
p.write32(AOP + 0x8808, msg1 & 0xffffffff)
p.write32(AOP + 0x880c, (msg1 >> 32) & 0xffffffff)

snap("post-send")

log("polling 5s for OUTBOX response...")
deadline = time.time() + 5
last_seen = None
got_hello = False
while time.time() < deadline:
    ob = p.read32(AOP + 0x8114)
    cs = p.read32(AOP + 0x48)
    if (ob, cs) != last_seen:
        t = 5 - (deadline - time.time())
        log(f"  t={t:.2f}s OB_CTRL={ob:#x} CS={cs:#x}")
        last_seen = (ob, cs)
    if not (ob & (1 << 17)):  # not EMPTY
        ob0 = p.read32(AOP + 0x8830) | (p.read32(AOP + 0x8834) << 32)
        ob1 = p.read32(AOP + 0x8838) | (p.read32(AOP + 0x883c) << 32)
        typ = (ob0 >> 52) & 0xff
        log(f"  *** OUTBOX msg: msg0={ob0:#x} msg1={ob1:#x} TYPE={typ:#x} ***")
        got_hello = True
        break
    time.sleep(0.05)

snap("final")

# Check IB state: did FW consume our INBOX msg?
ib = p.read32(AOP + 0x8110)
log(f"IB state: {ib:#x}  (WPTR={(ib>>8)&0xf}, RPTR={(ib>>12)&0xf}, FIFO={(ib>>20)&0xf})")

if got_hello:
    log("*** AOP RESPONDED! SUCCESS ***")
else:
    log("no response; FW still stuck")

os._exit(0)
