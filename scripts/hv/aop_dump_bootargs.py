#!/usr/bin/env python3
"""Dump all AOP bootargs keys and values."""
import os, pathlib, sys
ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))
os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port
iface = UartInterface()
p = M1N1Proxy(iface, debug=False)
bootstrap_port(iface, p)
u = ProxyUtils(p, heap_size=32 * 1024 * 1024)

from m1n1.fw.aop.base import AOPBase
ab = AOPBase(u)

addr, size = ab._bootargs_span
print(f"bootargs @ {addr:#x} size={size:#x}", flush=True)

args = ab.read_bootargs()
for k, v in args.items():
    print(f"  {k:4s} = {v.hex()} ({len(v)}B)")

os._exit(0)
