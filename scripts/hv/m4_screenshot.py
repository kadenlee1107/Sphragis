#!/usr/bin/env python3
"""Grab the M4's framebuffer over the m1n1 USB-CDC proxy and save as PNG.

Works against stock or patched m1n1 that's currently running and
listening on /dev/ttyACM1 as the proxy endpoint — i.e. NOT while
`run_guest.py` / `batos_hv_interactive.py` is holding the proxy in
HV mode (the proxy can only serve one client at a time).

Outputs:
  /tmp/m4_screen.ppm   raw
  /tmp/m4_screen.png   3x-downsized PNG

Pixel format: Apple M4 uses ARGB2101010 (30 bpp packed in 32-bit
words). We scale 10-bit channels down to 8-bit for the PNG.

Usage:
  sg dialout -c "/usr/bin/python3 scripts/hv/m4_screenshot.py"
"""
import sys
import pathlib
import time
import subprocess

M1N1_ROOT = pathlib.Path(__file__).resolve().parents[2] / "external/m1n1"
sys.path.insert(0, str(M1N1_ROOT / "proxyclient"))

from m1n1.proxy import *
from m1n1.proxyutils import *

import os
os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")

iface = UartInterface()
p = M1N1Proxy(iface, debug=False)
bootstrap_port(iface, p)

# M4 / T8132 observed framebuffer geometry:
FB_BASE = 0x103e0050000
WIDTH   = 3024
HEIGHT  = 1964
STRIDE  = WIDTH * 4   # 30-bpp packed into 32-bit words
SIZE    = STRIDE * HEIGHT

print(f"[m4-ss] reading {SIZE/1024/1024:.1f} MiB from FB at 0x{FB_BASE:x}...",
      flush=True)
t0 = time.time()
data = p.iface.readmem(FB_BASE, SIZE)
t1 = time.time()
print(f"[m4-ss] read done in {t1-t0:.1f}s "
      f"({SIZE/(t1-t0)/1024:.0f} KiB/s)", flush=True)

print("[m4-ss] decoding ARGB2101010 → RGB888...", flush=True)
out = bytearray(WIDTH * HEIGHT * 3)
oi = 0
for i in range(0, SIZE, 4):
    w = int.from_bytes(data[i:i+4], "little")
    r = (w >> 20) & 0x3ff
    g = (w >> 10) & 0x3ff
    b =  w         & 0x3ff
    out[oi]   = r >> 2
    out[oi+1] = g >> 2
    out[oi+2] = b >> 2
    oi += 3

ppm = pathlib.Path("/tmp/m4_screen.ppm")
png = pathlib.Path("/tmp/m4_screen.png")
with open(ppm, "wb") as f:
    f.write(f"P6\n{WIDTH} {HEIGHT}\n255\n".encode())
    f.write(bytes(out))
print(f"[m4-ss] wrote {ppm} ({ppm.stat().st_size/1024/1024:.1f} MiB)",
      flush=True)

# Downsize to PNG for easy sharing.
scale = int(os.environ.get("M4_SS_SCALE", "3"))
res = subprocess.run(
    ["ffmpeg", "-y", "-loglevel", "warning",
     "-i", str(ppm),
     "-vf", f"scale=iw/{scale}:ih/{scale}",
     "-update", "1", "-frames:v", "1", str(png)],
    capture_output=True, text=True,
)
if res.returncode == 0:
    print(f"[m4-ss] wrote {png} "
          f"({png.stat().st_size/1024:.0f} KiB, 1/{scale} scale)",
          flush=True)
else:
    print(f"[m4-ss] ffmpeg failed: {res.stderr}", flush=True)
