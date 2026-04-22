#!/usr/bin/env python3
"""Dump all ADT properties of /arm-io/aop and /arm-io/aop/iop-aop-nub
for clues about M4's boot protocol."""
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

def dump(path):
    print(f"\n=== {path} ===", flush=True)
    try:
        n = u.adt[path]
    except Exception as e:
        print(f"  ERR: {e}")
        return
    for attr in dir(n):
        if attr.startswith("_"): continue
        if attr in ("children", "AAPL,phandle", "build_adt", "items", "get_reg",
                    "get_ranges", "find_chosen", "getprop", "lookup"):
            continue
        try:
            v = getattr(n, attr)
        except Exception as e:
            continue
        if callable(v): continue
        if isinstance(v, bytes):
            if len(v) <= 64:
                try:
                    s = v.rstrip(b"\x00").decode("ascii")
                    print(f"  {attr}: {v.hex()} ({s!r})")
                except:
                    print(f"  {attr}: {v.hex()}")
            else:
                print(f"  {attr}: {len(v)}B  [{v[:32].hex()}...]")
        elif isinstance(v, (list, tuple)):
            if len(v) <= 16:
                print(f"  {attr}: {list(v)}")
            else:
                print(f"  {attr}: ({len(v)}) {list(v)[:8]}...")
        else:
            print(f"  {attr}: {v}")

for path in ("/arm-io/aop", "/arm-io/aop/iop-aop-nub",
             "/arm-io/mtp", "/arm-io/mtp/iop-mtp-nub"):
    dump(path)

# For the aop-nub, read memory at region-base and around bootargs span
try:
    nub = u.adt["/arm-io/aop/iop-aop-nub"]
    rb = nub.region_base
    print(f"\nAOP nub region_base = {rb:#x}", flush=True)
    # Offset 0x22c = bootargs addr offset, 0x230 = size, 0x234+ = dram addrs
    for off in (0x100, 0x200, 0x220, 0x224, 0x228, 0x22c, 0x230, 0x234, 0x238,
                0x23c, 0x240, 0x244, 0x248, 0x24c, 0x250):
        try:
            v = p.read32(rb + off)
            if v != 0:
                print(f"  +{off:#06x} = {v:#010x}")
        except Exception as e:
            print(f"  +{off:#06x} ERR {e}")
except Exception as e:
    print(f"nub probe fail: {e}")

# Check the iop-nub / region-base for MTP too
try:
    mnub = u.adt["/arm-io/mtp/iop-mtp-nub"]
    rb = mnub.region_base
    print(f"\nMTP nub region_base = {rb:#x}", flush=True)
    for off in (0x22c, 0x230, 0x234, 0x238, 0x23c, 0x240):
        v = p.read32(rb + off)
        print(f"  +{off:#06x} = {v:#010x}")
except Exception as e:
    print(f"mnub fail: {e}")

os._exit(0)
