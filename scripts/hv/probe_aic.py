#!/usr/bin/env python3
"""One-shot: probe ADT for /arm-io/aic + AOP/MTP interrupts to plan AIC mask."""
import os, pathlib, sys, struct
ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

iface = UartInterface()
p = M1N1Proxy(iface, debug=False)
bootstrap_port(iface, p)
u = ProxyUtils(p, heap_size=8 * 1024 * 1024)

def dump_node(path):
    try:
        n = u.adt[path]
    except KeyError:
        print(f"{path}: NOT FOUND")
        return
    print(f"{path}:")
    # reg
    try:
        for i in range(4):
            try:
                base, sz = n.get_reg(i)
                print(f"  reg[{i}] base={base:#x} size={sz:#x}")
            except Exception:
                break
    except Exception as e:
        print(f"  reg err: {e}")
    # compatible + interrupts + ranges
    for prop in ("compatible","interrupts","interrupt-parent","AAPL,phandle",
                 "aic-iack-offset","extintrcfg-stride","intmaskset-stride",
                 "intmaskclear-stride","aic-ext-intr-cfg","name"):
        v = getattr(n, prop, None)
        if v is None:
            continue
        if isinstance(v, bytes):
            try:
                s = v.rstrip(b"\x00").decode("ascii")
            except Exception:
                s = v.hex()
            print(f"  {prop}: {s!r}")
        elif isinstance(v, (list, tuple)):
            if len(v) <= 32:
                print(f"  {prop}: {list(v)}")
            else:
                print(f"  {prop}: ({len(v)} items) {list(v)[:8]}...")
        else:
            print(f"  {prop}: {v}")

for path in ("/arm-io/aic", "/arm-io/aop", "/arm-io/mtp",
             "/arm-io/aop/iop-aop-nub", "/arm-io/mtp/iop-mtp-nub",
             "/arm-io/dart-aop", "/arm-io/dart-mtp"):
    dump_node(path)

# AIC regs
try:
    aic_base, _ = u.adt["/arm-io/aic"].get_reg(0)
    print(f"\nAIC regs @ {aic_base:#x}:")
    for off, label in [(0x0, "ID/VER?"), (0x4, "NR_IRQ"), (0x10, "RST"),
                       (0x14, "CFG"), (0x28, "RR_DELAY"), (0x30, "CLUSTER_EN"),
                       (0x3c, "SOME_CNT")]:
        v = p.read32(aic_base + off)
        print(f"  +{off:#06x} [{label:12s}] = {v:#010x}")
except Exception as e:
    print(f"AIC probe err: {e}")

os._exit(0)
