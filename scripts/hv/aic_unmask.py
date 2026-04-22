#!/usr/bin/env python3
"""New theory: AOP FW is WAITING for AIC-delivered IRQ that's MASKED.

On ARM+AIC: IRQ_CFG.TARGET=0 = destination CPU 0 = AP CPU 0 usually.
But in some configs, AIC can deliver the IRQ to the AOP's OWN internal
CPU (coprocessor). If we UNMASK the IRQ at AIC, it'd be delivered to
whatever target is configured.

Actions:
  1. Snap AIC HW_STATE, MASK_SET, IRQ_CFG for AOP's IRQs.
  2. Clear mask for AOP IRQs (write 1 to MASK_CLR bits).
  3. See if CS changes.
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

AIC = 0x381000000
AIC_IRQ_CFG = 0x10000
AIC_MASK_SET = 0x14400
AIC_MASK_CLR = 0x14600
AIC_HW_STATE = 0x14800
AIC_EVENT    = 0x40000

AOP = 0x390600000
AOP_IRQS = [434, 433, 436, 435]
DART_AOP = [457]

def log(m): print(f"[aic-um] {m}", flush=True)

def snap(tag):
    cc = p.read32(AOP + 0x44)
    cs = p.read32(AOP + 0x48)
    ib = p.read32(AOP + 0x8110)
    ob = p.read32(AOP + 0x8114)
    b14 = p.read32(AOP + 0xb14)
    log(f"[{tag}] CC={cc:#x} CS={cs:#x} IB={ib:#x} OB={ob:#x} b14={b14:#x}")

def aic(tag):
    log(f"[{tag}] AIC state:")
    for irq in AOP_IRQS + DART_AOP:
        ro = (irq >> 5) * 4
        hw = p.read32(AIC + AIC_HW_STATE + ro)
        ms = p.read32(AIC + AIC_MASK_SET + ro)
        cfg = p.read32(AIC + AIC_IRQ_CFG + irq * 4)
        log(f"  irq {irq} (word{irq//32} bit{irq&31}): HW={hw:#x} MASK={ms:#x} CFG={cfg:#x}")

snap("current")
aic("before")

# Drain any pending events
log("draining AIC events...")
for i in range(8):
    ev = p.read32(AIC + AIC_EVENT)
    if ev == 0:
        log(f"  event[{i}]=0"); break
    log(f"  event[{i}]={ev:#x}")

# Try: clear mask for each AOP IRQ
log("clearing mask for AOP IRQs (MASK_CLR)...")
for irq in AOP_IRQS:
    ro = (irq >> 5) * 4
    bit = 1 << (irq & 31)
    p.write32(AIC + AIC_MASK_CLR + ro, bit)
    ms_after = p.read32(AIC + AIC_MASK_SET + ro)
    log(f"  irq {irq} cleared bit {bit:#x}; MASK={ms_after:#x}")

time.sleep(0.2)

snap("post-unmask")
aic("after-unmask")

# Drain events again
log("draining post-unmask...")
for i in range(16):
    ev = p.read32(AIC + AIC_EVENT)
    if ev == 0:
        log(f"  event[{i}]=0"); break
    die = (ev >> 24) & 0xff
    typ = (ev >> 16) & 0xff
    num = ev & 0xffff
    log(f"  event[{i}]={ev:#x} (die={die} type={typ} num={num})")

snap("final")

# If CS is still 0x48, try nudging the ASC by writing a test value to INBOX
# (already has SetIOPPower queued, let's just poll briefly)
log("waiting 2s for FW to process INBOX or emit OUTBOX...")
deadline = time.time() + 2
while time.time() < deadline:
    ob = p.read32(AOP + 0x8114)
    if not (ob & (1 << 17)):  # not empty
        log(f"  OUTBOX has msg! OB={ob:#x}")
        ob0 = p.read32(AOP + 0x8830) | (p.read32(AOP + 0x8834) << 32)
        ob1 = p.read32(AOP + 0x8838) | (p.read32(AOP + 0x883c) << 32)
        log(f"  OUT0={ob0:#x} OUT1={ob1:#x}")
        break
    time.sleep(0.1)
else:
    log("  no OUTBOX activity in 2s")
snap("final-poll")

os._exit(0)
