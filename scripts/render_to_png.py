#!/usr/bin/env python3
"""Run `render <url>` in Bat_OS, capture the base64-encoded BGRA dump
on serial, decode it to a PNG.

Usage:
    python3 scripts/render_to_png.py [url]

Default URL: file:///bin/hello.html

Outputs:
    logs/qemu-tests/render-<ts>.log  — full serial transcript
    logs/qemu-tests/render-<ts>.png  — the rendered framebuffer
"""
import base64, pexpect, re, socket, struct, subprocess, sys, time
from pathlib import Path
from datetime import datetime

ROOT = Path("/Users/kadenlee/Bat_OS")
TS = datetime.now().strftime("%Y%m%d-%H%M%S")
LOG = ROOT / f"logs/qemu-tests/render-{TS}.log"
PNG = ROOT / f"logs/qemu-tests/render-{TS}.png"
LOG.parent.mkdir(parents=True, exist_ok=True)
PROMPT = rb"bat_os\s*>\s*"

URL = sys.argv[1] if len(sys.argv) > 1 else "file:///bin/hello.html"

kernel_bin = ROOT / "target/aarch64-unknown-none/release/bat_os.bin"
initrd     = ROOT / "target/aarch64-unknown-none/release/chromium_initrd.bin"

# Refresh bat_os.bin if stale
elf = ROOT / "target/aarch64-unknown-none/release/bat_os"
if not kernel_bin.exists() or kernel_bin.stat().st_mtime < elf.stat().st_mtime:
    import shutil
    rust_objcopy = Path.home() / ".rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/rust-objcopy"
    if not rust_objcopy.exists():
        rust_objcopy = Path(shutil.which("llvm-objcopy") or "/opt/homebrew/Cellar/llvm/22.1.3/bin/llvm-objcopy")
    print(f"[render] objcopy …")
    subprocess.run([str(rust_objcopy), "-O", "binary", str(elf), str(kernel_bin)], check=True)

daemon = subprocess.Popen(
    ["python3", str(ROOT / "scripts" / "batcaved.py")],
    stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT,
)
for _ in range(40):
    try: socket.create_connection(("127.0.0.1", 9999), timeout=0.3).close(); break
    except OSError: time.sleep(0.2)

args = ["qemu-system-aarch64",
        "-accel", "hvf",
        "-machine", "virt,gic-version=3",
        "-cpu", "host",
        "-m", "4G",
        "-display", "none",
        "-serial", "mon:stdio",
        "-kernel", str(kernel_bin),
        "-initrd", str(initrd)]
fp = open(LOG, "wb")
c = pexpect.spawn(args[0], args[1:], timeout=120, logfile=fp, encoding=None)
try:
    c.expect(PROMPT, timeout=60)
    c.sendline(f"render {URL}".encode())
    # The render dump can be a couple hundred KB of base64. Wait
    # for the END marker explicitly so we know the whole image is in.
    c.expect(rb"=== RENDER-END ===", timeout=120)
    c.expect(PROMPT, timeout=10)
finally:
    c.terminate(force=True); fp.close()
    daemon.terminate()
    try: daemon.wait(timeout=3)
    except subprocess.TimeoutExpired: daemon.kill()

# Decode the captured base64 stream
raw = open(LOG, "rb").read().decode("utf-8", "replace")
m = re.search(r"=== RENDER-BEGIN (\d+)x(\d+) ===\s*\n(.*?)\n\s*=== RENDER-END ===",
              raw, re.DOTALL)
if not m:
    print(f"[render] FAILED — no RENDER-BEGIN/END markers in {LOG}")
    sys.exit(1)
W, H = int(m.group(1)), int(m.group(2))
b64_text = "".join(m.group(3).split())  # strip whitespace + newlines
bgra = base64.b64decode(b64_text)
expected_bytes = W * H * 4
if len(bgra) < expected_bytes:
    print(f"[render] WARNING: got {len(bgra)} bytes, expected {expected_bytes}")
    bgra = bgra + b"\x00" * (expected_bytes - len(bgra))
elif len(bgra) > expected_bytes:
    bgra = bgra[:expected_bytes]

# Convert BGRA -> RGBA for PNG
def bgra_to_rgba(buf):
    out = bytearray(len(buf))
    for i in range(0, len(buf), 4):
        out[i  ] = buf[i+2]  # R <- B-position-? Actually framebuffer is
        out[i+1] = buf[i+1]  # G <- G
        out[i+2] = buf[i  ]  # B <- R-position-? swap below
        out[i+3] = buf[i+3]  # A
    return bytes(out)
# The kernel writes ARGB8888 packed as a u32 little-endian. In bytes
# that's [B, G, R, A]. So channel order in the byte stream is BGRA.
rgba = bgra_to_rgba(bgra)

# Hand-roll a PNG (no PIL dependency).
import zlib
def png_chunk(tag, data):
    crc = zlib.crc32(tag + data)
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)
sig = b"\x89PNG\r\n\x1a\n"
ihdr = struct.pack(">IIBBBBB", W, H, 8, 6, 0, 0, 0)  # 8-bit RGBA
# Pre-pend a 0 filter byte to each row
filt = bytearray()
stride = W * 4
for y in range(H):
    filt.append(0)
    filt.extend(rgba[y*stride : (y+1)*stride])
idat = zlib.compress(bytes(filt), 6)
with open(PNG, "wb") as f:
    f.write(sig)
    f.write(png_chunk(b"IHDR", ihdr))
    f.write(png_chunk(b"IDAT", idat))
    f.write(png_chunk(b"IEND", b""))

print(f"[render] wrote {PNG} ({W}x{H}, {len(bgra)} raw bytes)")
print(f"[render] log:   {LOG}")
print(f"[render] open with: open {PNG}")
