import sys, pathlib
sys.path.insert(0, "/home/kaden-lee/code/Bat_OS/external/m1n1/proxyclient")
from m1n1.setup import *
# First confirm proxy alive with a safe read
v1 = p.read64(0x211e00000 + 0x20020)
print(f"PCPU +0x20020 OK = {v1:#018x}")
# Now probe +0x200f8 (the problem register)
try:
    v2 = p.read64(0x211e00000 + 0x200f8)
    print(f"PCPU +0x200f8 OK = {v2:#018x}")
except Exception as e:
    print(f"PCPU +0x200f8 SErrored: {type(e).__name__}: {e}")
