#!/usr/bin/env python3
"""Single-pass MTP SRAM write-protection test.

Hypothesis (from probe_mtp_power_state.py + SRAM-vs-Mach-O diff):
iBoot already stages the A5PH __TEXT segment into 0x394c00000.
The 15:45 session's write to 0x394c00100 faulted because that
address contains LIVE CODE — a write attempting to overwrite
execute-only SRAM. __DATA (0x394c5f000) and __OS_LOG (0x10005640000)
are NOT staged (live SRAM reads zeros; Mach-O has content).

If this hypothesis is right, writes to the __DATA region should
succeed, and we can finish firmware staging by copying only
__DATA + __OS_LOG (skipping __TEXT).

Safety: three write attempts, from lowest to highest risk. Each is
wrapped with a short timeout. A fault on any call wedges m1n1 —
but at that point we know where the boundary is.
"""
import os
import pathlib
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port


TESTS = [
    # (name, addr, reason)
    ("__OS_LOG", 0x10005640000,
     "DRAM — should ALWAYS work (belt-and-suspenders sanity test)"),
    ("__DATA",   0x00394c5f000,
     "MTP SRAM, zero region in Mach-O — critical hypothesis test"),
    ("__DATA+0x100", 0x00394c5f100,
     "Same region, different offset — confirms not just-at-base"),
]

MAGIC = 0xdeadbeef


def main():
    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)

    # baseline read on all three addresses, to confirm current state
    print("=== Baseline reads (should all work) ===")
    for name, addr, _ in TESTS:
        try:
            before = p.read32(addr)
            print(f"  {name:16s} @ {addr:#014x}: read32 -> {before:#010x}")
        except Exception as e:
            print(f"  {name:16s} @ {addr:#014x}: READ FAILED: {e!r}")
            return 1

    print()
    print("=== Single-word write test ===")
    print(f"  writing magic {MAGIC:#010x} to each target and reading back")
    print()

    for name, addr, reason in TESTS:
        print(f"--- [{name}] @ {addr:#014x} ({reason}) ---")
        print(f"  write32({addr:#x}, {MAGIC:#010x}) ...")
        sys.stdout.flush()
        t0 = time.time()
        try:
            p.write32(addr, MAGIC)
            dt = time.time() - t0
            print(f"  write OK in {dt*1000:.0f} ms")
        except Exception as e:
            dt = time.time() - t0
            print(f"  write FAILED in {dt*1000:.0f} ms: {e!r}")
            print(f"  >>> m1n1 proxy likely wedged; aborting further tests <<<")
            return 2
        try:
            after = p.read32(addr)
            print(f"  read32({addr:#x}) -> {after:#010x}", end="")
            if after == MAGIC:
                print(" ✓ (matches)")
            else:
                print(f" ✗ (expected {MAGIC:#010x})")
        except Exception as e:
            print(f"  readback FAILED: {e!r}")
            return 2
        print()

    print("=== ALL THREE WRITES SUCCEEDED ===")
    print("iBoot-only-stages-__TEXT hypothesis confirmed.")
    print("Next: update stage_mtp_firmware.py to stage __DATA + __OS_LOG only,")
    print("      then kick CPU_CONTROL.RUN=1.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
