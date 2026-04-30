#!/usr/bin/env python3
"""Render an HTML file using Bat_OS's native browser engine + decode the
base64 BGRA dump on serial back into a PNG.

Usage:
    python3 scripts/render_to_png.py [url]

Default URL: file:///bin/hello.html

Outputs:
    logs/qemu-tests/render-<timestamp>.log
    logs/qemu-tests/render-<timestamp>.png
"""
from __future__ import annotations

import base64
import re
import struct
import sys
import zlib
from pathlib import Path

# Make sure we can import from scripts/lib/ when invoked from any cwd.
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from scripts.lib.qemu_boot import boot, ROOT  # noqa: E402


URL = sys.argv[1] if len(sys.argv) > 1 else "file:///bin/hello.html"


def bgra_to_rgba(buf: bytes) -> bytes:
    """The kernel writes ARGB8888 packed as a u32 little-endian. In bytes
    that's [B, G, R, A]. PNGs want RGBA — swap channels 0 and 2."""
    out = bytearray(len(buf))
    for i in range(0, len(buf), 4):
        out[i  ] = buf[i + 2]
        out[i + 1] = buf[i + 1]
        out[i + 2] = buf[i]
        out[i + 3] = buf[i + 3]
    return bytes(out)


def write_png(path: Path, w: int, h: int, rgba: bytes) -> None:
    """Hand-rolled PNG writer (no PIL dependency)."""
    def chunk(tag, data):
        crc = zlib.crc32(tag + data)
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)

    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)  # 8-bit RGBA
    stride = w * 4
    filt = bytearray()
    for y in range(h):
        filt.append(0)  # filter byte: none
        filt.extend(rgba[y * stride : (y + 1) * stride])
    idat = zlib.compress(bytes(filt), 6)
    with open(path, "wb") as f:
        f.write(sig)
        f.write(chunk(b"IHDR", ihdr))
        f.write(chunk(b"IDAT", idat))
        f.write(chunk(b"IEND", b""))


def main() -> int:
    with boot(log_prefix="render", timeout=120) as session:
        session.run(f"render {URL}".encode())
        # The base64 dump is a couple hundred KB. Wait for END marker
        # explicitly so we know the image is complete.
        session.expect(rb"=== RENDER-END ===", timeout=120)
        session.expect_prompt(timeout=10)

    log = session.log  # captured by the boot harness's logfile
    raw = log.read_text(encoding="utf-8", errors="replace")

    m = re.search(
        r"=== RENDER-BEGIN (\d+)x(\d+) ===\s*\n(.*?)\n\s*=== RENDER-END ===",
        raw, re.DOTALL,
    )
    if not m:
        print(f"[render] FAILED — no RENDER-BEGIN/END markers in {log}",
              file=sys.stderr)
        return 1
    w, h = int(m.group(1)), int(m.group(2))
    b64 = "".join(m.group(3).split())  # drop newlines/whitespace
    bgra = base64.b64decode(b64)

    expected = w * h * 4
    if len(bgra) < expected:
        print(f"[render] WARNING: got {len(bgra)} bytes, expected {expected}",
              file=sys.stderr)
        bgra = bgra + b"\x00" * (expected - len(bgra))
    elif len(bgra) > expected:
        bgra = bgra[:expected]

    rgba = bgra_to_rgba(bgra)
    png = log.with_suffix(".png")
    write_png(png, w, h, rgba)

    print(f"[render] wrote {png} ({w}x{h}, {len(bgra)} raw bytes)")
    print(f"[render] log:   {log}")
    print(f"[render] open with: open {png}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
