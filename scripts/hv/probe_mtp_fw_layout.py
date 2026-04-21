#!/usr/bin/env python3
"""Dump MTP ASC firmware layout from the live M4 ADT.

Run after m1n1 is chainloaded (any version — just reads ADT). Prints
segment-ranges triplets from the MTP node + segment-names from the
iop-mtp-nub child so we know where each Mach-O segment (__TEXT /
__DATA / __OS_LOG) needs to land in physical memory before mtp.boot().
"""
import os
import pathlib
import struct
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port


def parse_segment_ranges(raw):
    """Decode segment-ranges as 32-byte records: phys(u64) + iova(u64)
    + remap(u64) + size(u32) + 4 pad. Matches the pattern used by
    m1n1's ISP fw loader (proxyclient/m1n1/fw/isp/isp_base.py:305)."""
    segs = []
    for i in range(len(raw) // 32):
        seg = raw[i * 32:(i + 1) * 32]
        phys, iova, remap, size = struct.unpack("<QQQI4x", seg)
        segs.append({"phys": phys, "iova": iova, "remap": remap, "size": size})
    return segs


def main():
    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    mtp = u.adt["/arm-io/mtp"]
    print(f"MTP ASC reg[0]: phys={mtp.get_reg(0)[0]:#x} "
          f"size={mtp.get_reg(0)[1]:#x}")
    print(f"MTP compatible: {list(getattr(mtp, 'compatible', []))}")

    for attr in ("segment-ranges", "segment-names", "fw-shape",
                 "fw-region", "iommu-parent", "role"):
        val = getattr(mtp, attr, None)
        if isinstance(val, bytes):
            print(f"  MTP.{attr} ({len(val)} bytes): {val[:64].hex()}")
        else:
            print(f"  MTP.{attr}: {val!r}")

    try:
        nub = u.adt["/arm-io/mtp/iop-mtp-nub"]
    except Exception:
        nub = None
    if nub is not None:
        print("iop-mtp-nub:")
        for attr in ("segment-ranges", "segment-names", "coredump-enable",
                     "uuid", "KDebugCoreID"):
            val = getattr(nub, attr, None)
            if isinstance(val, bytes) and attr != "uuid":
                print(f"  nub.{attr} ({len(val)} bytes): {val.hex()}")
            else:
                print(f"  nub.{attr}: {val!r}")

        sr = getattr(nub, "segment-ranges", None)
        names = getattr(nub, "segment-names", "")
        if isinstance(names, bytes):
            names = names.decode("ascii", errors="replace").strip("\x00")
        if sr:
            segs = parse_segment_ranges(sr)
            name_list = names.split(";") if names else []
            print(f"\nParsed {len(segs)} segments (names={name_list}):")
            for i, s in enumerate(segs):
                nm = name_list[i] if i < len(name_list) else "?"
                print(f"  [{i}] {nm:>10}  phys={s['phys']:#014x}  "
                      f"iova={s['iova']:#014x}  remap={s['remap']:#014x}  "
                      f"size={s['size']:#x} ({s['size']} bytes)")

    # dockchannel-mtp (MTP host transport)
    try:
        dc = u.adt["/arm-io/dockchannel-mtp"]
        print(f"\ndockchannel-mtp:")
        for i in range(6):
            try:
                r = dc.get_reg(i)
                if r:
                    print(f"  reg[{i}] phys={r[0]:#x} size={r[1]:#x}")
            except Exception:
                break
    except Exception:
        pass


if __name__ == "__main__":
    main()
