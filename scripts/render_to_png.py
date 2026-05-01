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
    # The kernel emits one RENDER-BEGIN/END block per page (STUMP #89).
    # We don't know up front how many pages we'll see, so we wait for
    # the shell prompt to come back instead of for a single END marker.
    with boot(log_prefix="render", timeout=180) as session:
        session.run(f"render {URL}".encode())
        session.expect_prompt(timeout=180)

    log = session.log  # captured by the boot harness's logfile
    raw = log.read_text(encoding="utf-8", errors="replace")

    # Match every RENDER-BEGIN block. Header is either of:
    #   === RENDER-BEGIN <W>x<H> ===                       (single-page legacy)
    #   === RENDER-BEGIN <W>x<H> page=<i>/<n> ===          (paginated)
    pattern = re.compile(
        r"=== RENDER-BEGIN (?P<w>\d+)x(?P<h>\d+)"
        r"(?:\s+page=(?P<idx>\d+)/(?P<total>\d+))?\s+===\s*\n"
        r"(?P<b64>.*?)\n\s*=== RENDER-END ===",
        re.DOTALL,
    )
    matches = list(pattern.finditer(raw))
    if not matches:
        print(f"[render] FAILED — no RENDER-BEGIN/END markers in {log}",
              file=sys.stderr)
        return 1

    paths = []
    for m in matches:
        w = int(m.group("w"))
        h = int(m.group("h"))
        idx_s = m.group("idx")
        b64 = "".join(m.group("b64").split())  # drop newlines/whitespace
        bgra = base64.b64decode(b64)

        expected = w * h * 4
        if len(bgra) < expected:
            print(f"[render] WARNING: got {len(bgra)} bytes, expected {expected}",
                  file=sys.stderr)
            bgra = bgra + b"\x00" * (expected - len(bgra))
        elif len(bgra) > expected:
            bgra = bgra[:expected]

        rgba = bgra_to_rgba(bgra)

        # Single-page (legacy) → foo.png. Paginated → foo.p1.png, foo.p2.png ...
        if idx_s is None or len(matches) == 1:
            png = log.with_suffix(".png")
        else:
            png = log.with_suffix(f".p{int(idx_s) + 1}.png")
        write_png(png, w, h, rgba)
        paths.append((png, w, h, len(bgra)))

    for png, w, h, n in paths:
        print(f"[render] wrote {png} ({w}x{h}, {n} raw bytes)")
    print(f"[render] log:   {log}")
    if len(paths) == 1:
        print(f"[render] open with: open {paths[0][0]}")
    else:
        print(f"[render] open with: open {' '.join(str(p[0]) for p in paths)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
