#!/usr/bin/env python3
"""Walk the /arm-io/aop ADT subtree to see what iBoot left us —
firmware binary, segment table, region spans, etc. AOP rtkit_boot
timed out in the first attempt so we need to load FW ourselves.
"""
import sys, pathlib
M1N1 = pathlib.Path(__file__).resolve().parents[2] / "external/m1n1/proxyclient"
sys.path.insert(0, str(M1N1))
from m1n1.setup import *

def dump_node(path, depth=0):
    try:
        n = u.adt[path]
    except (KeyError, Exception) as e:
        print(f"{'  '*depth}{path}: NOT FOUND ({e})")
        return
    print(f"{'  '*depth}{path}:")
    try:
        compat = str(n.compatible)
        print(f"{'  '*depth}  compatible: {compat}")
    except Exception:
        pass
    for prop in n._properties.keys():
        try:
            v = getattr(n, prop)
            if isinstance(v, bytes):
                if len(v) > 64:
                    print(f"{'  '*depth}  {prop}: <{len(v)} bytes> first16={v[:16].hex()}")
                else:
                    print(f"{'  '*depth}  {prop}: {v.hex() if len(v) <= 32 else v[:16].hex()+'...'}")
            else:
                s = repr(v)
                if len(s) > 100:
                    s = s[:100] + "..."
                print(f"{'  '*depth}  {prop}: {s}")
        except Exception as e:
            print(f"{'  '*depth}  {prop}: <err: {e}>")


print("=== /arm-io/aop ===")
dump_node("/arm-io/aop")
print()
print("=== /arm-io/aop/iop-aop-nub ===")
dump_node("/arm-io/aop/iop-aop-nub")
print()
print("=== /arm-io/dart-aop ===")
dump_node("/arm-io/dart-aop")
