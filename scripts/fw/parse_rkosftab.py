#!/usr/bin/env python3
"""Parse an Apple RTKit OS firmware table (`rkosftab`).

Layout (observed on M4 J604_MtpFirmware.bin, 0.9 MB):

  offset  size   content
  0x00    0x20   header (mostly zero; `ffffffff` at 0x08)
  0x20    8      magic bytes "rkosftab"
  0x28    8      count / version — observed `02 00 00 00 00 00 00 00`
                 (count=2 low u32, 0 pad)
  0x30    N*16   entries: tag[4] + offset[4] + size[4] + pad[4],
                 each offset is a FILE offset pointing at the
                 section payload, each size is bytes.

J604 sections observed:
  tag='A5PH' offset=0x50       size=0xcf000  — RTKit Mach-O (main)
  tag='iokt' offset=0xcf050    size=0xd1e7   — IOKit personality dict

The "A5PH" payload at offset 0x50 is a Mach-O (`cffaedfe` magic =
MH_MAGIC_64) with __TEXT/__DATA segments that get loaded into the
MTP ASC's SRAM before CPU_CONTROL.RUN=1.

Usage:
  python3 scripts/fw/parse_rkosftab.py firmware/mtp/J604_MtpFirmware.bin
"""
import pathlib
import struct
import sys


MAGIC = b"rkosftab"


def parse(data):
    """Parse an rkosftab blob. Returns list of (tag, offset, size, payload)."""
    magic_off = data.find(MAGIC)
    if magic_off < 0:
        raise ValueError("no rkosftab magic found")
    # Skip magic (8) + count/version (8) to reach entries.
    entries_off = magic_off + 16
    entries = []
    cursor = entries_off
    while cursor + 16 <= len(data):
        tag_bytes = data[cursor:cursor + 4]
        if any(b == 0 for b in tag_bytes) and tag_bytes != b"\x00\x00\x00\x00":
            # Partial null — end of table.
            break
        if tag_bytes == b"\x00\x00\x00\x00":
            break
        if not all(32 <= b < 127 for b in tag_bytes):
            break
        tag = tag_bytes.decode("ascii")
        offset, size = struct.unpack("<II", data[cursor + 4:cursor + 12])
        # 4 bytes pad follow.
        if offset == 0 or size == 0 or offset + size > len(data):
            # A5PH on J604 has weird reserved4 that parses as offset — skip
            # invalid entries rather than raising.
            cursor += 16
            continue
        payload = data[offset:offset + size]
        entries.append((tag, offset, size, payload))
        cursor += 16
    return entries


def main(argv):
    if len(argv) != 2:
        sys.stderr.write(f"usage: {argv[0]} firmware.bin\n")
        return 1
    data = pathlib.Path(argv[1]).read_bytes()
    entries = parse(data)
    print(f"total blob size: {len(data)} ({len(data):#x}) bytes")
    print(f"sections: {len(entries)}")
    for tag, off, size, payload in entries:
        magic = payload[:4].hex()
        head = "".join(chr(b) if 32 <= b < 127 else "." for b in payload[:32])
        print(f"  {tag!r}: file_off={off:#x} size={size:#x} ({size} bytes) "
              f"head_magic={magic} head={head!r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
