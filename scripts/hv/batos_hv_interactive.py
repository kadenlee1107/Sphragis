#!/usr/bin/env python3
"""Run Bat_OS under m1n1's hypervisor and drive an interactive shell
session over /dev/ttyACM2 (the USB-CDC vuart secondary endpoint).

This script combines m1n1's run_guest.py functionality with a
pyserial-backed reader/writer thread on the vuart endpoint, so the
host tty driver can't drop the connection between reads and the
Bat_OS guest can receive bytes we type.

Prerequisites:
  * Patched m1n1.macho already chainloaded on the Mac (see
    `docs/SESSION_JOURNAL.md` for the chainload command).
  * udev rule in `/etc/udev/rules.d/99-m1n1.rules` granting dialout
    group access to the m1n1 uartproxy device.
  * User in group dialout (or wrap this with `sg dialout -c '...'`).

Usage:
  sg dialout -c "/usr/bin/python3 scripts/hv/batos_hv_interactive.py"
  # (in interactive mode, Ctrl+D to exit)

Options via env var:
  BATOS_HV_STIMULUS="help\\r"   # send this string after the prompt
                                # appears; if unset, read stdin live.
"""
import sys, os, time, termios, threading, pathlib

M1N1_ROOT = pathlib.Path(__file__).resolve().parents[2] / "external/m1n1"
sys.path.insert(0, str(M1N1_ROOT / "proxyclient"))

import serial
from m1n1.proxy import *
from m1n1.proxyutils import *
from m1n1.utils import *
from m1n1.hv import HV

BAT_OS_BINARY = pathlib.Path(__file__).resolve().parents[2] / \
    "target/bat_os_apple.bin"

def configure_vuart_raw(vuart):
    """Set raw termios on the vuart tty: no echo, no canonical, no
    opost. Without this, Ubuntu's tty layer defaults echo CRLF from
    the guest back as `^M^J` on the input side, creating an echo loop
    that looks like Bat_OS is malfunctioning. It isn't."""
    vuart.dtr = True   # m1n1 DWC3 needs DTR for pipe ready
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

def main():
    stimulus = os.environ.get("BATOS_HV_STIMULUS", "")
    if stimulus:
        stimulus = stimulus.encode().decode("unicode_escape").encode()

    vuart = serial.Serial(
        "/dev/ttyACM2",
        baudrate=115200,
        timeout=0.1,
        rtscts=False, xonxoff=False, dsrdtr=False,
    )
    configure_vuart_raw(vuart)

    sent = [False]
    def reader():
        buf = b""
        while True:
            try:
                d = vuart.read(1024)
            except serial.SerialException as e:
                print(f"\n[vuart] serial exception: {e}", flush=True)
                return
            if d:
                sys.stdout.buffer.write(d)
                sys.stdout.buffer.flush()
                buf += d
                if stimulus and not sent[0] and b"bat_os>" in buf:
                    sent[0] = True
                    time.sleep(0.3)
                    print(f"\n[vuart] >>> sending {stimulus!r}",
                          flush=True)
                    vuart.write(stimulus)
                    vuart.flush()
            time.sleep(0.01)
    threading.Thread(target=reader, daemon=True).start()

    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size = 128 * 1024 * 1024)
    hv = HV(iface, p, u)
    hv.init()

    print(f"[host] loading {BAT_OS_BINARY}", flush=True)
    hv.load_raw(BAT_OS_BINARY.read_bytes(), 0)

    print("[host] calling hv.start() — Bat_OS takes over now. "
          "Ctrl+C to stop.", flush=True)
    try:
        hv.start()
    except KeyboardInterrupt:
        pass
    except Exception as e:
        print(f"\n[host] hv.start() raised: {e}", flush=True)

    print("\n[host] HV exited — draining vuart for 3s", flush=True)
    time.sleep(3)
    vuart.close()

if __name__ == "__main__":
    main()
