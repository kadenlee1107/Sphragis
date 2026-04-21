#!/usr/bin/env python3
"""Dump MTP ASC power-state + ADT diagnostics (READ-ONLY).

Goal: figure out why p.write32(0x394c00100, ...) faults inside m1n1's
EL2 while p.read32 on the same addresses works. Four candidate causes
(PMGR gate, SPRR label, missing DART mapping, iBoot guard) — this
probe collects the state needed to pick one without touching memory.

Safe to re-run on a fresh m1n1. Never writes.

Usage:
  /usr/bin/python3 scripts/hv/probe_mtp_power_state.py [--sweep]

Flags:
  --sweep   Also sweep the MTP ASC register aperture (reg[0]) in
            16-byte groups, printing non-zero rows. Slower (~1 min).
"""
import argparse
import os
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port


def dump_node(u, path, depth=0):
    """Iterate all ADT properties of a node, pretty-print each.
    Matches the style of scripts/hv/probe_aop_firmware.py so output
    diffs cleanly against the AOP dump."""
    pad = "  " * depth
    try:
        n = u.adt[path]
    except (KeyError, Exception) as e:
        print(f"{pad}{path}: NOT FOUND ({e})")
        return None
    print(f"{pad}{path}:")
    try:
        compat = list(getattr(n, "compatible", []))
        print(f"{pad}  compatible: {compat}")
    except Exception:
        pass
    for prop in n._properties.keys():
        try:
            v = getattr(n, prop)
        except Exception as e:
            print(f"{pad}  {prop}: <err: {e}>")
            continue
        if isinstance(v, bytes):
            if len(v) <= 64:
                print(f"{pad}  {prop} ({len(v)}B): {v.hex()}")
            else:
                print(f"{pad}  {prop} ({len(v)}B): {v[:64].hex()}...")
        else:
            s = repr(v)
            if len(s) > 200:
                s = s[:200] + "..."
            print(f"{pad}  {prop}: {s}")
    return n


def sweep_aperture(p, base, size, stride=16):
    """Read `size` bytes starting at `base` as u32 words, group by
    `stride`. Print rows where any word is non-zero."""
    print(f"\n=== ASC aperture sweep {base:#x}..{base + size:#x} (stride={stride}) ===")
    non_zero = 0
    rows_printed = 0
    for off in range(0, size, stride):
        row = []
        any_nz = False
        for w in range(0, stride, 4):
            try:
                val = p.read32(base + off + w)
            except Exception as e:
                print(f"  {base + off + w:#x}: read err: {e!r}")
                return
            row.append(val)
            if val:
                any_nz = True
        if any_nz:
            non_zero += 1
            if rows_printed < 256:
                hex_words = " ".join(f"{v:08x}" for v in row)
                print(f"  {base + off:#x}: {hex_words}")
                rows_printed += 1
    print(f"(swept {size // stride} rows; {non_zero} non-zero; printed up to 256)")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--sweep", action="store_true",
                    help="Also sweep MTP ASC reg[0] (slow ~1 min)")
    args = ap.parse_args()

    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=32 * 1024 * 1024)

    print("=== ADT: /arm-io/mtp (full property dump) ===")
    mtp = dump_node(u, "/arm-io/mtp")

    print("\n=== ADT: /arm-io/mtp/iop-mtp-nub ===")
    dump_node(u, "/arm-io/mtp/iop-mtp-nub")

    print("\n=== ADT: /arm-io/pmgr (top-level attrs only) ===")
    try:
        pmgr = u.adt["/arm-io/pmgr"]
        print("/arm-io/pmgr:")
        for prop in pmgr._properties.keys():
            # power/clock state tables are huge — just show size + first bytes
            try:
                v = getattr(pmgr, prop)
            except Exception as e:
                print(f"  {prop}: <err: {e}>")
                continue
            if isinstance(v, bytes):
                if len(v) <= 32:
                    print(f"  {prop} ({len(v)}B): {v.hex()}")
                else:
                    print(f"  {prop} ({len(v)}B): {v[:32].hex()}...")
            else:
                s = repr(v)
                if len(s) > 160:
                    s = s[:160] + "..."
                print(f"  {prop}: {s}")
    except Exception as e:
        print(f"  /arm-io/pmgr dump err: {e!r}")

    # MTP ASC reg[0] — the register aperture itself. CPU_CONTROL / CPU_STATUS
    # / IMPL reg live here. Even if the SRAM aperture is gated, this one
    # should be readable (we already know reg ops work; the symptom is
    # *write* faults).
    if mtp is not None:
        print("\n=== MTP ASC reg[0] — key registers ===")
        try:
            mtp_base = mtp.get_reg(0)[0]
            print(f"  reg[0] base: {mtp_base:#x}")
            # StandardASC layout (external/m1n1/proxyclient/m1n1/fw/asc/base.py):
            #   CPU_CONTROL at +0x0044
            #   CPU_STATUS  at +0x0048
            # ASC IMPL regs are in +0x0400..+0x0600 region on M1/M2.
            regs = [
                (0x0000, "IRQ_CTRL"),
                (0x0004, "IRQ_STATUS"),
                (0x0008, "IRQ_MASK"),
                (0x000c, "IRQ_unk0"),
                (0x0040, "CPU_unk0"),
                (0x0044, "CPU_CONTROL"),
                (0x0048, "CPU_STATUS"),
                (0x004c, "CPU_unk1"),
                (0x0050, "CPU_unk2"),
                (0x0100, "IMPL_0x100"),
                (0x0104, "IMPL_0x104"),
                (0x0108, "IMPL_0x108"),
                (0x010c, "IMPL_0x10c"),
                (0x0110, "IMPL_0x110"),
                (0x0400, "IMPL_0x400"),
                (0x0404, "IMPL_0x404"),
                (0x0408, "IMPL_0x408"),
                (0x040c, "IMPL_0x40c"),
                (0x0500, "IMPL_0x500"),
                (0x0504, "IMPL_0x504"),
                (0x0600, "IMPL_0x600"),
                (0x0700, "IMPL_0x700"),
                (0x8000, "INBOX0_lo"),
                (0x8004, "INBOX0_hi"),
                (0x8008, "INBOX1_lo"),
                (0x800c, "INBOX1_hi"),
                (0x8110, "INBOX_CTRL"),
                (0x8114, "OUTBOX_CTRL"),
                (0x8830, "OUTBOX0_lo"),
                (0x8834, "OUTBOX0_hi"),
                (0x8838, "OUTBOX1_lo"),
                (0x883c, "OUTBOX1_hi"),
            ]
            for off, name in regs:
                try:
                    v = p.read32(mtp_base + off)
                    print(f"  [+{off:#06x}] {name:20s} = {v:#010x}")
                except Exception as e:
                    print(f"  [+{off:#06x}] {name:20s} = <read err: {e}>")
                    break
        except Exception as e:
            print(f"  reg read err: {e!r}")

    # Additional probe: confirm the SRAM read-pattern we already
    # documented (4-byte 'b #+0x244' at 0x394c00000, OS_LOG strings at
    # 0x10005640000) — sanity check the target is still in the state
    # we expect at this power-cycle.
    print("\n=== SRAM read-back (sanity) ===")
    for addr, name, nbytes in [
        (0x394c00000,   "__TEXT[0]",  16),
        (0x394c00100,   "__TEXT+0x100", 16),  # the write-target that faults
        (0x394c5f000,   "__DATA[0]",   16),
        (0x10005640000, "__OS_LOG[0]", 64),
    ]:
        try:
            data = iface.readmem(addr, nbytes)
            print(f"  {name:14s} @ {addr:#014x}: {data.hex()}")
            if nbytes >= 32:
                # Print trailing ASCII if it looks like a format string
                try:
                    s = data.decode("ascii", errors="replace")
                    printable = "".join(c if 32 <= ord(c) < 127 else "." for c in s)
                    print(f"  {' ':14s}   ASCII: {printable!r}")
                except Exception:
                    pass
        except Exception as e:
            print(f"  {name:14s} @ {addr:#014x}: <err: {e}>")

    if args.sweep and mtp is not None:
        try:
            mtp_base, mtp_size = mtp.get_reg(0)
            sweep_aperture(p, mtp_base, mtp_size, stride=16)
        except Exception as e:
            print(f"sweep err: {e!r}")

    print("\n(done — no writes attempted)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
