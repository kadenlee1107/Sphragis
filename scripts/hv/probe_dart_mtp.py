"""Check dart-mtp error / status registers for translation faults.
The FW is hung post-SetIOPPower. If it faulted on a DART translation,
the DART hw latches the fault info — iova, stream, type."""
import sys, os
sys.path.insert(0, "external/m1n1/proxyclient")
os.environ["M1N1DEVICE"] = "/dev/ttyACM1"
from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

iface = UartInterface()
p = M1N1Proxy(iface)
bootstrap_port(iface, p)
u = ProxyUtils(p)

dart_base = u.adt["/arm-io/dart-mtp"].get_reg(0)[0]
print(f"dart-mtp base: {dart_base:#x}")

# Common Apple DART error reg offsets (t8110/t6020 variants):
#   +0x100: TCR[0]
#   +0x104: TCR[1]
#   +0x14c: ERR_STATUS
#   +0x150: ERR_ADDR_LO
#   +0x154: ERR_ADDR_HI
# Also check register mapping in m1n1
print("\n=== DART reg dump 0x0..0x400 ===")
for off in range(0, 0x400, 4):
    v = p.read32(dart_base + off)
    if v:
        print(f"  [+{off:#06x}] = {v:#010x}")

# More targeted: known DART error/fault regs
print("\n=== Suspected fault/error regs ===")
for off, name in [
    (0x100, "TCR[0]"),
    (0x104, "TCR[1]"),
    (0x140, "ERR_0x140"),
    (0x144, "ERR_0x144"),
    (0x148, "ERR_0x148"),
    (0x14c, "ERR_STATUS"),
    (0x150, "ERR_ADDR_LO"),
    (0x154, "ERR_ADDR_HI"),
    (0x158, "ERR_0x158"),
    (0x160, "ERR_IRQ_MASK"),
]:
    v = p.read32(dart_base + off)
    print(f"  [+{off:#06x}] {name:20s} = {v:#010x}")

# Look for the iova space used by FW — scan page-table entries
print("\n=== DART TTBR / page table state (if accessible) ===")
# TTBR is usually at +0x200 region
for off in range(0x200, 0x240, 4):
    v = p.read32(dart_base + off)
    if v:
        print(f"  [+{off:#06x}] = {v:#010x}")

# Also check the MTP ASC's own IRQ status
mtp_base = 0x394600000
print(f"\n=== MTP IRQ/status (non-mailbox) ===")
for off, name in [(0x0, "IRQ_CTRL"), (0x4, "IRQ_STATUS"), (0x8, "IRQ_MASK"),
                  (0xc, "IRQ_ack"), (0x50, "?"), (0x54, "?"),
                  (0x444, "IMPL_0x444"), (0x450, "?")]:
    v = p.read32(mtp_base + off)
    print(f"  [+{off:#06x}] {name:14s} = {v:#010x}")

# Examine __DATA now — has FW written to it further?
print("\n=== __DATA at +0x0, +0x1000, +0x10000 (stack canary), +0x20000 ===")
for off in (0, 0x100, 0x1000, 0x4000, 0x10000, 0x20000, 0x40000):
    sample = iface.readmem(0x394c5f000 + off, 32)
    if any(b for b in sample):
        print(f"  [+{off:#x}] = {sample.hex()}")

os._exit(0)
