#!/usr/bin/env python3
"""Receive a Bat_OS `screen` command dump on /dev/ttyACM2 and
reconstruct it as a PNG.

Works mid-HV-session: sit listening on the vuart, tell the Bat_OS
shell to `screen <scale>`, parse the SCREEN_BEGIN header + hex pixel
rows + SCREEN_END, decode ARGB2101010 → RGB888, save to
/tmp/batos_screen.png.

Expects the shell session (batos_hv_interactive.py with BATOS_KEEP_FB=1)
to be running and holding the vuart open via DTR. This script is a
passive reader on the same /dev/ttyACM2.

Usage:
  # terminal 1 — start Bat_OS with KEEP_FB (holds ttyACM1 + ttyACM2):
  BATOS_KEEP_FB=1 BATOS_HV_STIMULUS='screen 4' sg dialout -c \\
      "/usr/bin/python3 scripts/hv/batos_hv_interactive.py" \\
      > /tmp/hv.log 2>&1 &
  # terminal 2 — catch the dump on the same vuart:
  sg dialout -c "/usr/bin/python3 scripts/hv/m4_screen_dump.py"

Because only one reader can open /dev/ttyACM2 at a time, the more
robust flow is: run batos_hv_interactive.py WITHOUT the vuart reader
side (set BATOS_HV_NO_READER=1 TBD) OR just run this script directly
with stimulus built in.

This version is SELF-CONTAINED: it runs batos_hv_interactive's HV
setup, fires 'screen\\r', and reads the dump from the same process.
"""
import sys, os, time, termios, pathlib, re, struct, threading

M1N1_ROOT = pathlib.Path(__file__).resolve().parents[2] / "external/m1n1"
sys.path.insert(0, str(M1N1_ROOT / "proxyclient"))

import serial
from m1n1.proxy import *
from m1n1.proxyutils import *
from m1n1.utils import *
from m1n1.hv import HV

BAT_OS_BINARY = pathlib.Path(__file__).resolve().parents[2] / \
    "target/bat_os_apple.bin"

SCALE = int(os.environ.get("M4_SCREEN_SCALE", "4"))
OUT_PNG = pathlib.Path(os.environ.get("M4_SCREEN_OUT", "/tmp/batos_screen.png"))
OUT_PPM = OUT_PNG.with_suffix(".ppm")


def configure_vuart_raw(vuart):
    vuart.dtr = True
    vuart.rts = False
    fd = vuart.fileno()
    a = termios.tcgetattr(fd)
    a[0] &= ~(termios.BRKINT | termios.ICRNL | termios.INPCK
              | termios.ISTRIP | termios.IXON)
    a[1] &= ~termios.OPOST
    a[2] = (a[2] & ~(termios.CSIZE | termios.PARENB)) | termios.CS8
    a[3] &= ~(termios.ECHO | termios.ECHONL | termios.ICANON
              | termios.ISIG | termios.IEXTEN)
    termios.tcsetattr(fd, termios.TCSANOW, a)


def parser_thread(vuart, result, done, ready_event):
    """Reads bytes from vuart, scans for SCREEN_BEGIN .. SCREEN_END,
    buffers everything in between, returns it via `result[0]`."""
    buf = bytearray()
    state = "pre"
    header = {}
    rows = []
    while not done.is_set():
        try:
            chunk = vuart.read(65536)
        except serial.SerialException as e:
            print(f"[dump] serial error: {e}", flush=True)
            return
        if not chunk:
            continue
        buf.extend(chunk)
        while True:
            nl = buf.find(b"\n")
            if nl < 0:
                break
            line = bytes(buf[:nl])
            del buf[:nl+1]
            if state == "pre":
                if line.startswith(b"SCREEN_BEGIN "):
                    for kv in line[len(b"SCREEN_BEGIN "):].split():
                        k, _, v = kv.partition(b"=")
                        header[k.decode()] = v.decode()
                    state = "body"
                    print(f"[dump] header={header}", flush=True)
                else:
                    sys.stdout.buffer.write(line + b"\n")
                    sys.stdout.flush()
                    # Trigger the stimulator once Bat_OS's self-test
                    # replay completes and the real prompt is next.
                    if b"launching apple shell" in line \
                       or b"[selftest] replay complete" in line \
                       or b"No display" in line:
                        ready_event.set()
            elif state == "body":
                if line == b"SCREEN_END":
                    result.append((header, rows))
                    done.set()
                    return
                if re.fullmatch(rb"[0-9a-fA-F]+", line):
                    rows.append(line.decode())
                else:
                    sys.stdout.buffer.write(b"[interleaved] " + line + b"\n")
                    sys.stdout.flush()


def decode_argb2101010_rows(header, rows):
    w = int(header["w"])
    h = int(header["h"])
    out = bytearray(w * h * 3)
    oi = 0
    rows_got = len(rows)
    for y in range(h):
        if y >= rows_got:
            # missing row — fill black
            oi += w * 3
            continue
        hexrow = rows[y]
        if len(hexrow) < w * 8:
            hexrow = hexrow.ljust(w * 8, "0")
        for x in range(w):
            word = int(hexrow[x*8:(x+1)*8], 16)
            r = (word >> 20) & 0x3ff
            g = (word >> 10) & 0x3ff
            b =  word        & 0x3ff
            out[oi]   = r >> 2
            out[oi+1] = g >> 2
            out[oi+2] = b >> 2
            oi += 3
    return bytes(out), w, h


def main():
    vuart = serial.Serial("/dev/ttyACM2", 115200, timeout=0.2,
                          rtscts=False, xonxoff=False, dsrdtr=False)
    configure_vuart_raw(vuart)

    result = []
    done = threading.Event()
    ready = threading.Event()
    t = threading.Thread(target=parser_thread,
                         args=(vuart, result, done, ready),
                         daemon=True)
    t.start()

    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)
    hv = HV(iface, p, u)
    hv.init()
    print(f"[dump] loading {BAT_OS_BINARY}", flush=True)
    hv.load_raw(BAT_OS_BINARY.read_bytes(), 0)

    # Schedule the screen command AFTER the parser sees Bat_OS reach
    # the interactive shell (post self-test replay).
    def stimulate():
        print("[dump] waiting for 'replay complete' marker...", flush=True)
        if not ready.wait(timeout=60.0):
            print("[dump] never saw ready marker, sending anyway", flush=True)
        # Small settle before typing.
        time.sleep(0.5)
        print(f"[dump] sending 'screen {SCALE}\\r'", flush=True)
        vuart.write(f"screen {SCALE}\r".encode())
        vuart.flush()
    threading.Thread(target=stimulate, daemon=True).start()

    print("[dump] calling hv.start() — Mac resets in ~100 s under KEEP_FB",
          flush=True)
    try:
        hv.start()
    except Exception as e:
        print(f"[dump] hv.start() raised: {e}", flush=True)

    # Give reader a bit to catch final bytes.
    if not done.is_set():
        done.wait(timeout=3.0)
    vuart.close()

    if not result:
        print("[dump] no SCREEN_BEGIN/END seen — capture incomplete", flush=True)
        sys.exit(1)

    header, rows = result[0]
    print(f"[dump] decoding {len(rows)} rows...", flush=True)
    rgb, w, h = decode_argb2101010_rows(header, rows)
    with open(OUT_PPM, "wb") as f:
        f.write(f"P6\n{w} {h}\n255\n".encode())
        f.write(rgb)
    # Convert to PNG via ffmpeg for easier sharing.
    import subprocess
    res = subprocess.run(
        ["ffmpeg", "-y", "-loglevel", "warning", "-i", str(OUT_PPM),
         "-update", "1", "-frames:v", "1", str(OUT_PNG)],
        capture_output=True, text=True,
    )
    if res.returncode == 0:
        print(f"[dump] wrote {OUT_PNG} "
              f"({OUT_PNG.stat().st_size/1024:.0f} KiB)",
              flush=True)
    else:
        print(f"[dump] ffmpeg failed: {res.stderr}", flush=True)


if __name__ == "__main__":
    main()
