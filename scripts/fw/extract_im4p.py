#!/usr/bin/env python3
"""Extract Apple Image4 Payload (.im4p) wrappers → raw payload bytes.

.im4p is an ASN.1 DER SEQUENCE: magic("IM4P") + 4-char type + version +
OCTET-STRING payload (optional keybag / signature fields follow and are
ignored here). Apple wraps ASC / RTKit firmware and input-device config
blobs in this format on macOS under /System/Volumes/Preboot/.../Firmware/.

Usage:
  python3 scripts/fw/extract_im4p.py firmware/mtp/*.im4p

Output: sibling .bin file next to each .im4p.
"""
import pathlib
import sys


def unwrap_im4p(data):
    """Parse an im4p. Returns (type_4cc, version, payload_bytes)."""
    assert data[0] == 0x30, "not an ASN.1 SEQUENCE"
    i = 1
    if data[i] & 0x80:
        nlen = data[i] & 0x7f
        i += 1 + nlen
    else:
        i += 1

    def read_str(off):
        assert data[off] == 0x16, f"expected IA5String at {off:#x}"
        length = data[off + 1]
        return data[off + 2:off + 2 + length].decode("ascii"), off + 2 + length

    def read_octet(off):
        assert data[off] == 0x04, f"expected OCTET STRING at {off:#x}"
        off += 1
        if data[off] & 0x80:
            nlen = data[off] & 0x7f
            length = int.from_bytes(data[off + 1:off + 1 + nlen], "big")
            off += 1 + nlen
        else:
            length = data[off]
            off += 1
        return data[off:off + length], off + length

    magic, i = read_str(i)
    assert magic == "IM4P", f"bad magic: {magic!r}"
    typ, i = read_str(i)
    ver, i = read_str(i)
    payload, _ = read_octet(i)
    return typ, ver, payload


def main(argv):
    if len(argv) < 2:
        sys.stderr.write(f"usage: {argv[0]} file.im4p [file.im4p ...]\n")
        return 1
    for arg in argv[1:]:
        p = pathlib.Path(arg)
        data = p.read_bytes()
        typ, ver, payload = unwrap_im4p(data)
        out = p.with_suffix(".bin")
        out.write_bytes(payload)
        print(f"{p.name}: type={typ!r} ver={ver!r} -> {out.name} "
              f"({len(payload)} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
