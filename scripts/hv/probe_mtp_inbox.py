import sys, os, time
sys.path.insert(0, "external/m1n1/proxyclient")
os.environ["M1N1DEVICE"] = "/dev/ttyACM1"
from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import bootstrap_port

iface = UartInterface()
p = M1N1Proxy(iface)
bootstrap_port(iface, p)

base = 0x394600000
# Read state right after the previous boot attempt
# INBOX_CTRL + INBOX data (what we sent)
print("=== INBOX state (what host sent) ===")
print(f"  INBOX_CTRL [+0x8110] = {p.read32(base + 0x8110):#010x}")
print(f"  INBOX0     [+0x8800] = {p.read64(base + 0x8800):#018x}")
print(f"  INBOX1     [+0x8808] = {p.read64(base + 0x8808):#018x}")

# OUTBOX state (what FW sent)
print("\n=== OUTBOX state ===")
print(f"  OUTBOX_CTRL[+0x8114] = {p.read32(base + 0x8114):#010x}")
print(f"  OUTBOX0    [+0x8830] = {p.read64(base + 0x8830):#018x}")
print(f"  OUTBOX1    [+0x8838] = {p.read64(base + 0x8838):#018x}")
# Re-read to see if it advances
print(f"  OUTBOX0 (2){' '*4}  = {p.read64(base + 0x8830):#018x}")
print(f"  OUTBOX1 (2){' '*4}  = {p.read64(base + 0x8838):#018x}")

# Dump the area near INBOX_CTRL/OUTBOX_CTRL — maybe there are other
# relevant regs. 0x8100..0x8200 window:
print("\n=== 0x8100..0x8180 ===")
for off in range(0x8100, 0x8180, 16):
    row = [f"{p.read32(base + off + w):08x}" for w in (0, 4, 8, 12)]
    print(f"  [+{off:#06x}] {' '.join(row)}")

# Send a test message to INBOX manually and watch for response
import struct
print("\n=== Sending Mgmt_Ping (type=3) manually ===")
# Mgmt_Ping body = 0, TYPE=3, EP=0
# mgmt messages use the upper bits for TYPE, lower for fields
# Mgmt_Ping: TYPE at bits 59..52. value = 3 << 52
msg0 = 3 << 52
msg1 = 0  # EP=0
print(f"  writing INBOX0={msg0:#x} INBOX1={msg1:#x}")
p.write64(base + 0x8800, msg0)
p.write64(base + 0x8808, msg1)
time.sleep(0.5)
print(f"  after 500ms: INBOX_CTRL={p.read32(base + 0x8110):#x}")
print(f"  OUTBOX_CTRL = {p.read32(base + 0x8114):#x}")
for i in range(3):
    o0 = p.read64(base + 0x8830)
    o1 = p.read64(base + 0x8838)
    print(f"  read #{i+1}: OUTBOX0={o0:#x}  OUTBOX1={o1:#x}")

# CPU state
cc = p.read32(base + 0x44)
cs = p.read32(base + 0x48)
print(f"\nCPU_CONTROL={cc:#x}  CPU_STATUS={cs:#x}")

import os as _os
_os._exit(0)
