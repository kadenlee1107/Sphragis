#!/usr/bin/env python3
"""ADT walk for CPU cluster bases + current PSTATE register state
on M4. Needed to add T8132 support to m1n1's cpufreq_fixup."""
import sys, pathlib
M1N1 = pathlib.Path(__file__).resolve().parents[2] / "external/m1n1/proxyclient"
sys.path.insert(0, str(M1N1))
from m1n1.setup import *

# Walk ADT looking for cpu-cluster entries
def dump(path):
    try:
        n = u.adt[path]
    except Exception as e:
        print(f"{path}: err {e}")
        return
    print(f"{path}:")
    try:
        compat = str(n.compatible)
        print(f"  compat: {compat}")
    except Exception:
        pass
    for prop in n._properties.keys():
        try:
            v = getattr(n, prop)
            if isinstance(v, bytes):
                if len(v) > 64:
                    print(f"  {prop}: <{len(v)} bytes> first32={v[:32].hex()}")
                else:
                    print(f"  {prop}: {v.hex()}")
            else:
                s = repr(v)
                if len(s) > 140:
                    s = s[:140] + "..."
                print(f"  {prop}: {s}")
        except Exception:
            pass

# Walk the top-level /cpus node and /cpus/cpu-clusters if present
print("=== walking /cpus ===")
try:
    for child in u.adt["/cpus"]:
        print(f"  /cpus/{child.name}")
except Exception as e:
    print(f"err: {e}")

# Typical cluster node naming on M1/M2
for guess in ("/cpus/cpu-clusters/cluster0",
              "/cpus/cpu-clusters/cluster1",
              "/cpus/cluster0",
              "/cpus/cluster1",
              "/arm-io/cpufreq-bluetooth",
              "/soc/cpus/cluster0",
              "/soc/cpus/cluster1"):
    dump(guess)

print()
print("=== reading live PSTATE registers at expected M1/M2 bases ===")
CLUSTER_PSTATE = 0x20020  # m1n1 src/cpufreq.c
for (name, base) in [("ECPU @ 0x210e00000", 0x210e00000),
                     ("PCPU @ 0x211e00000", 0x211e00000)]:
    try:
        v = p.read64(base + CLUSTER_PSTATE)
        print(f"  {name}: PSTATE @ 0x{base + CLUSTER_PSTATE:x} = 0x{v:016x}")
    except Exception as e:
        print(f"  {name}: err {e}")
