#!/usr/bin/env python3
"""Poke AIC while AOP is in stuck state to see if anything moves.

Assumes AOP already booted from prior script and is sitting at CS=0x48
(IRQ pending). Test:
  1. Read AIC HW_STATE for AOP IRQs — are any of them actually asserted?
  2. If asserted, try AIC MASK_SET → does CS bit 2 toggle?
  3. Drain AIC EVENT — see what events are queued.
"""
import os, pathlib, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

AIC = 0x381000000
AIC_IRQ_CFG = 0x10000
AIC_SW_SET = 0x14000
AIC_MASK_SET = 0x14400
AIC_MASK_CLR = 0x14600
AIC_HW_STATE = 0x14800
AIC_EVENT = 0x40000

AOP_IRQS = [434, 433, 436, 435]
MTP_IRQS = [1114, 1113, 1116, 1115]
DART_AOP_IRQS = [457]
AOP_BASE = 0x390600000

iface = UartInterface()
p = M1N1Proxy(iface, debug=False)
bootstrap_port(iface, p)
u = ProxyUtils(p, heap_size=8 * 1024 * 1024)

def log(m): print(f"[aic-poke] {m}", flush=True)


def snap_asc(base, tag):
    cc = p.read32(base + 0x44)
    cs = p.read32(base + 0x48)
    ib = p.read32(base + 0x8110)
    ob = p.read32(base + 0x8114)
    b14 = p.read32(base + 0xb14)
    log(f"[{tag}] CC={cc:#x} CS={cs:#x} IB={ib:#x} OB={ob:#x} +b14={b14:#x}")


def aic_state(label):
    log(f"=== {label} ===")
    seen = set()
    for irq in AOP_IRQS + DART_AOP_IRQS:
        ro = (irq >> 5) * 4
        key = ro
        if key in seen: continue
        seen.add(key)
        hw = p.read32(AIC + AIC_HW_STATE + ro)
        ms = p.read32(AIC + AIC_MASK_SET + ro)
        sw = p.read32(AIC + AIC_SW_SET + ro)
        log(f"  AIC reg word {irq//32}(+{ro:#x}): HW_STATE={hw:#x} MASK_SET={ms:#x} SW_SET={sw:#x}")
    # IRQ_CFG for each AOP irq
    for irq in AOP_IRQS + DART_AOP_IRQS:
        cfg_addr = AIC + AIC_IRQ_CFG + irq * 4
        cfg = p.read32(cfg_addr)
        log(f"  IRQ_CFG[{irq}] = {cfg:#x} (target nibble bits 3:0)")


snap_asc(AOP_BASE, "ASC now")
aic_state("BEFORE")

log("draining AIC EVENT up to 16x...")
for i in range(16):
    ev = p.read32(AIC + AIC_EVENT)
    if ev == 0:
        log(f"  event[{i}] = 0 (drained)")
        break
    log(f"  event[{i}] = {ev:#x} (die={(ev>>24)&0xff} type={(ev>>16)&0xff} num={ev&0xffff})")

snap_asc(AOP_BASE, "post-drain")

log("masking AOP IRQs...")
for irq in AOP_IRQS:
    ro = (irq >> 5) * 4
    bit = 1 << (irq & 31)
    p.write32(AIC + AIC_MASK_SET + ro, bit)
    ms = p.read32(AIC + AIC_MASK_SET + ro)
    log(f"  irq {irq} wrote bit {bit:#x}; MASK_SET now {ms:#x}")

snap_asc(AOP_BASE, "post-mask")

log("draining again...")
for i in range(8):
    ev = p.read32(AIC + AIC_EVENT)
    if ev == 0:
        log(f"  event[{i}] = 0 (drained)")
        break
    log(f"  event[{i}] = {ev:#x}")

snap_asc(AOP_BASE, "final")

os._exit(0)
