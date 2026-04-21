"""Compare AOP (running) vs MTP (stopped) ASC register state.
Both are ascwrap-v6 on M4. AOP is always-on, booted by iBoot before
we arrive. MTP is what we've been trying to bring up.
"""
import sys, os
sys.path.insert(0, "external/m1n1/proxyclient")
os.environ["M1N1DEVICE"] = "/dev/ttyACM1"
from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import bootstrap_port

iface = UartInterface()
p = M1N1Proxy(iface)
bootstrap_port(iface, p)
from m1n1.proxyutils import ProxyUtils
u = ProxyUtils(p)

mtp_base = u.adt["/arm-io/mtp"].get_reg(0)[0]
try:
    aop_base = u.adt["/arm-io/aop"].get_reg(0)[0]
except Exception as e:
    print(f"no /arm-io/aop: {e}")
    import os as _os; _os._exit(1)

aop_compat = list(getattr(u.adt["/arm-io/aop"], "compatible", []))
mtp_compat = list(getattr(u.adt["/arm-io/mtp"], "compatible", []))
print(f"AOP @ {aop_base:#x}  compat={aop_compat}")
print(f"MTP @ {mtp_base:#x}  compat={mtp_compat}")
print()

regs = [
    (0x0044, "CPU_CONTROL"),
    (0x0048, "CPU_STATUS"),
    (0x0040, "CPU_unk0"),
    (0x0400, "IMPL_0x400"),
    (0x0444, "IMPL_0x444"),
    (0x080c, "unk_0x80c"),
    (0x8110, "INBOX_CTRL"),
    (0x8114, "OUTBOX_CTRL"),
]

print(f"{'reg':>16s}  {'AOP':>12s}  {'MTP':>12s}  same?")
for off, name in regs:
    a = p.read32(aop_base + off)
    m = p.read32(mtp_base + off)
    same = "=" if a == m else "DIFF"
    print(f"  {name:>14s}[+{off:#06x}]  {a:#010x}  {m:#010x}  {same}")

print()
print("=== OUTBOX0/OUTBOX1 (u64) ===")
for off, name in [(0x8800, "INBOX0"), (0x8808, "INBOX1"),
                  (0x8830, "OUTBOX0"), (0x8838, "OUTBOX1")]:
    a = p.read64(aop_base + off)
    m = p.read64(mtp_base + off)
    print(f"  {name:>8s}  AOP={a:#018x}  MTP={m:#018x}")

print()
print("=== Non-zero in AOP reg[0] 0x0..0x1000 ===")
for off in range(0, 0x1000, 4):
    v = p.read32(aop_base + off)
    if v:
        print(f"  [+{off:#06x}] = {v:#010x}")

import os as _os; _os._exit(0)
