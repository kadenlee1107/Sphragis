#!/usr/bin/env python3
"""Second-look probe after MTP ASC entered RUN+IDLE state.

We know:
  - __TEXT staged by iBoot at 0x394c00000 (matches Mach-O)
  - __DATA/__OS_LOG newly staged by us
  - CPU_CONTROL=0x10 (RUN=1), CPU_STATUS=0x6c (STOPPED=0, IDLE=1)
  - OUTBOX stays EMPTY — no Hello after 10 s

This read-only probe inspects things we haven't yet:
  - DART /arm-io/dart-mtp TCR/config — is iova translation enabled?
  - DART TTBR — is a page table installed?
  - The ASC's IMPL registers — does one of them hold a status code?
  - The mailbox state with a fresh read pattern.
  - Whether __DATA changed after the ASC started (FW writing to it?).
"""
import os
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port


def main():
    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=32 * 1024 * 1024)

    mtp = u.adt["/arm-io/mtp"]
    mtp_base = mtp.get_reg(0)[0]

    print(f"=== MTP ASC @ 0x{mtp_base:x} state ===")
    cc = p.read32(mtp_base + 0x0044)
    cs = p.read32(mtp_base + 0x0048)
    print(f"  CPU_CONTROL = {cc:#010x}   CPU_STATUS = {cs:#010x}")

    # IMPL registers. M1/M2 ASCs have runtime status latches around
    # 0x100..0x700. Print anything non-zero.
    print("\n  IMPL register scan (0x100..0x7fc):")
    any_nz = False
    for off in range(0x100, 0x800, 4):
        try:
            v = p.read32(mtp_base + off)
        except Exception:
            continue
        if v:
            print(f"    [+{off:#06x}] = {v:#010x}")
            any_nz = True
    if not any_nz:
        print("    (all zero)")

    # Mailbox + inbox
    print("\n  Mailbox state:")
    for off, name in [
        (0x8000, "INBOX0_lo"), (0x8004, "INBOX0_hi"),
        (0x8008, "INBOX1_lo"), (0x800c, "INBOX1_hi"),
        (0x8110, "INBOX_CTRL"), (0x8114, "OUTBOX_CTRL"),
        (0x8800, "INBOX0_64a"), (0x8808, "INBOX1_64a"),
        (0x8830, "OUTBOX0_lo"), (0x8834, "OUTBOX0_hi"),
        (0x8838, "OUTBOX1_lo"), (0x883c, "OUTBOX1_hi"),
    ]:
        v = p.read32(mtp_base + off)
        print(f"    [+{off:#06x}] {name:14s} = {v:#010x}")

    # DART /arm-io/dart-mtp — TCR + TTBR. If TRANSLATE_ENABLE==0 the
    # firmware can't fetch via iova, which could explain the hang.
    print("\n=== DART /arm-io/dart-mtp state ===")
    try:
        dart_node = u.adt["/arm-io/dart-mtp"]
        dart_base = dart_node.get_reg(0)[0]
        print(f"  dart base: {dart_base:#x}")
        # Typical DART t8110 regs: 0x0=???, 0x100=TCR0, 0x104=TCR1, ...
        # Read broad region.
        for off in range(0x0, 0x400, 4):
            v = p.read32(dart_base + off)
            if v:
                print(f"    [+{off:#06x}] = {v:#010x}")
    except Exception as e:
        print(f"  DART probe err: {e!r}")

    # Did FW write to __DATA after starting? Compare current to Mach-O.
    print("\n=== __DATA runtime change? ===")
    head_before = bytes.fromhex("00" * 16)  # staged value (all-zero header)
    head_now = iface.readmem(0x394c5f000, 64)
    print(f"  __DATA[0..64] now: {head_now.hex()}")
    print(f"  __DATA[0..64] at stage: {head_before.hex()}...")
    # Also look at __OS_LOG - MTP FW should write to this if running
    oslog_now = iface.readmem(0x10005ea0000, 64)
    print(f"  __OS_LOG[0..64] now: {oslog_now.hex()[:64]}")

    # Sample __DATA at mid-offsets (not just head)
    for off in (0x100, 0x1000, 0x10000, 0x40000, 0x60000):
        if off < 0x6c000:
            sample = iface.readmem(0x394c5f000 + off, 16)
            if any(b for b in sample):
                print(f"  __DATA[+{off:#x}] = {sample.hex()} (non-zero)")
            else:
                print(f"  __DATA[+{off:#x}] = zero")

    # Check CPU_unk regs near CPU_CONTROL
    print("\n  CPU_unk registers:")
    for off in (0x40, 0x4c, 0x50, 0x54, 0x58, 0x5c, 0x60):
        v = p.read32(mtp_base + off)
        print(f"    [+{off:#04x}] = {v:#010x}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
