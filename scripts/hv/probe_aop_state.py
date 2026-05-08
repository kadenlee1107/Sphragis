#!/usr/bin/env python3
"""Read AOP ASC state directly from stock m1n1 proxy — CPU control,
mailbox control, any pending message — so we can understand why
rtkit_boot failed."""
import sys
import pathlib
M1N1 = pathlib.Path(__file__).resolve().parents[2] / "external/m1n1/proxyclient"
sys.path.insert(0, str(M1N1))
from m1n1.setup import *

# /arm-io/aop base from the earlier probe = 15307112448 = 0x38E1C0000
# asc_init: cpu_base = reg[0], base = reg[0] + 0x8000
AOP_BASE = u.adt["/arm-io/aop"].get_reg(0)[0]
CPU_CTRL = 0x44
MBOX_BASE = AOP_BASE + 0x8000

ASC_MBOX_A2I_CTRL = 0x110
ASC_MBOX_I2A_CTRL = 0x114
ASC_MBOX_I2A_RECV0 = 0x830
ASC_MBOX_I2A_RECV1 = 0x838

print(f"AOP ASC base = 0x{AOP_BASE:x}")
print(f"CPU_CONTROL @ 0x{AOP_BASE + CPU_CTRL:x} = 0x{p.read32(AOP_BASE + CPU_CTRL):08x}")
print(f"A2I_CONTROL @ 0x{MBOX_BASE + ASC_MBOX_A2I_CTRL:x} = 0x{p.read32(MBOX_BASE + ASC_MBOX_A2I_CTRL):08x}")
print(f"I2A_CONTROL @ 0x{MBOX_BASE + ASC_MBOX_I2A_CTRL:x} = 0x{p.read32(MBOX_BASE + ASC_MBOX_I2A_CTRL):08x}")

# compare with SMC which works
SMC_BASE = u.adt["/arm-io/smc"].get_reg(0)[0]
SMC_MBOX = SMC_BASE + 0x8000
print()
print("--- SMC for comparison ---")
print(f"SMC ASC base = 0x{SMC_BASE:x}")
print(f"CPU_CONTROL @ 0x{SMC_BASE + CPU_CTRL:x} = 0x{p.read32(SMC_BASE + CPU_CTRL):08x}")
print(f"A2I_CONTROL @ 0x{SMC_MBOX + ASC_MBOX_A2I_CTRL:x} = 0x{p.read32(SMC_MBOX + ASC_MBOX_A2I_CTRL):08x}")
print(f"I2A_CONTROL @ 0x{SMC_MBOX + ASC_MBOX_I2A_CTRL:x} = 0x{p.read32(SMC_MBOX + ASC_MBOX_I2A_CTRL):08x}")
