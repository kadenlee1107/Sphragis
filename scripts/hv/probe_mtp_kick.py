"""FW read first INBOX message then stopped. Try: clear +0x0b14 (suspected
RPTR/status latch), poke different regs, see what triggers FW to consume more."""
import sys
import os
import time
sys.path.insert(0, "external/m1n1/proxyclient")
os.environ["M1N1DEVICE"] = "/dev/ttyACM1"
from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import bootstrap_port

iface = UartInterface()
p = M1N1Proxy(iface)
bootstrap_port(iface, p)

base = 0x394600000
print(f"pre: CC={p.read32(base+0x44):#x} CS={p.read32(base+0x48):#x}")
print(f"     +0x80c={p.read32(base+0x80c):#x}")
print(f"     +0xb14={p.read32(base+0xb14):#x}")
print(f"     INBOX_CTRL={p.read32(base+0x8110):#x}")
print(f"     OUTBOX_CTRL={p.read32(base+0x8114):#x}")

# Test 1: write 0 to +0xb14 — clear suspected status
print("\n--- test 1: clear +0xb14 ---")
p.write32(base + 0xb14, 0)
time.sleep(0.5)
print(f"  +0xb14 now={p.read32(base+0xb14):#x}")
print(f"  INBOX_CTRL={p.read32(base+0x8110):#x}")
print(f"  OUTBOX_CTRL={p.read32(base+0x8114):#x}")

# Test 2: write 0x100 (what FW had) back
print("\n--- test 2: write 0x100 back ---")
p.write32(base + 0xb14, 0x100)
time.sleep(0.5)
print(f"  +0xb14 now={p.read32(base+0xb14):#x}")
print(f"  INBOX_CTRL={p.read32(base+0x8110):#x}")
print(f"  OUTBOX_CTRL={p.read32(base+0x8114):#x}")

# Test 3: poke +0x080c
print("\n--- test 3: write +0x080c = 0 ---")
old_80c = p.read32(base + 0x80c)
p.write32(base + 0x80c, 0)
time.sleep(0.5)
print(f"  +0x80c was={old_80c:#x}, now={p.read32(base+0x80c):#x}")
print(f"  INBOX_CTRL={p.read32(base+0x8110):#x}")

# Test 4: send a different message type and see if RPTR moves
# Send Mgmt_StartEP (type=5, EP=0x1)
print("\n--- test 4: send Mgmt_StartEP(EP=1) ---")
msg0 = (5 << 52) | (0x1 << 32) | 2   # Mgmt_StartEP: EP at 39..32, FLAG at 1..0
msg1 = 0
p.write64(base + 0x8800, msg0)
p.write64(base + 0x8808, msg1)
time.sleep(1.0)
print(f"  INBOX_CTRL={p.read32(base+0x8110):#x}")
print(f"  OUTBOX_CTRL={p.read32(base+0x8114):#x}")
print(f"  +0xb14={p.read32(base+0xb14):#x}")
print(f"  CS={p.read32(base+0x48):#x}")

# Test 5: read all of 0xb00..0xb20 block to see what's in there
print("\n--- test 5: +0xb00..+0xb20 block ---")
for off in range(0xb00, 0xb20, 4):
    v = p.read32(base + off)
    print(f"  [+{off:#x}] = {v:#x}")

# Test 6: scan for any CHANGED regs
print("\n--- test 6: scan 0x0..0x1000 for new non-zero ---")
for off in range(0, 0x1000, 4):
    v = p.read32(base + off)
    if v:
        print(f"  [+{off:#x}] = {v:#x}")

os._exit(0)
