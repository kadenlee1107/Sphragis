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
  * udev rule granting dialout group access to m1n1 uartproxy.
  * User in group dialout (or wrap with `sg dialout -c '...'`).

Usage:
  # Interactive REPL — stdin bytes go to the guest, guest TX goes
  # to stdout. Ctrl+] to detach cleanly.
  sg dialout -c "/usr/bin/python3 scripts/hv/batos_hv_interactive.py"

  # Canned stimulus — just fire one command and print the response.
  # CR is appended automatically if missing; ';;' separates commands.
  BATOS_HV_STIMULUS="help" sg dialout -c \\
      "/usr/bin/python3 scripts/hv/batos_hv_interactive.py"

  # Multiple stimuli (separated by ';;' or newlines) with 500 ms gaps:
  BATOS_HV_STIMULUS="uname;;mem;;uptime" sg dialout -c ...

Output:
  * Bytes from the guest (ttyACM2 vuart) print to stdout live.
  * Bytes from the m1n1 proxy (ttyACM1) print prefixed with
    `TTY>` — that's m1n1's own library formatting.
  * Our host-side status lines are prefixed `[host]` / `[vuart]`
    and go to stderr so they don't pollute captured logs.
"""
import sys, os, time, termios, tty, threading, pathlib, select

M1N1_ROOT = pathlib.Path(__file__).resolve().parents[2] / "external/m1n1"
sys.path.insert(0, str(M1N1_ROOT / "proxyclient"))

import serial
from m1n1.proxy import *
from m1n1.proxyutils import *
from m1n1.utils import *
from m1n1.hv import HV

BAT_OS_BINARY = pathlib.Path(__file__).resolve().parents[2] / \
    "target/bat_os_apple.bin"

ESCAPE_BYTE = 0x1d   # Ctrl+] — same as miniterm convention.


def log(msg):
    sys.stderr.write(f"[host] {msg}\n")
    sys.stderr.flush()


def configure_vuart_raw(vuart):
    """Raw termios + DTR asserted. Without this, Ubuntu's tty layer
    echo+canonical settings create an infinite CRLF echo loop that
    looks exactly like Bat_OS is broken."""
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


def stimulus_sender(vuart, items):
    """Fire each byte-string in items after a short delay."""
    time.sleep(1.5)  # give the guest time to print the prompt
    for stim in items:
        sys.stderr.write(f"\n[vuart] >>> {stim!r}\n")
        sys.stderr.flush()
        vuart.write(stim)
        vuart.flush()
        time.sleep(0.8)


def vuart_reader(vuart):
    while True:
        try:
            d = vuart.read(1024)
        except serial.SerialException as e:
            sys.stderr.write(f"\n[vuart] serial exception: {e}\n")
            sys.stderr.flush()
            return
        if d:
            sys.stdout.buffer.write(d)
            sys.stdout.buffer.flush()
        time.sleep(0.005)


def stdin_forwarder(vuart, stop):
    """If stdin is a tty, put it in cbreak mode and forward each key
    to the guest. Ctrl+] detaches."""
    if not sys.stdin.isatty():
        return
    old = termios.tcgetattr(sys.stdin.fileno())
    try:
        tty.setcbreak(sys.stdin.fileno())
        while not stop.is_set():
            r, _, _ = select.select([sys.stdin], [], [], 0.1)
            if sys.stdin in r:
                b = os.read(sys.stdin.fileno(), 64)
                if not b:
                    break
                if ESCAPE_BYTE in b:
                    break
                try:
                    vuart.write(b)
                    vuart.flush()
                except Exception as e:
                    sys.stderr.write(f"[vuart] write failed: {e}\n")
                    break
    finally:
        termios.tcsetattr(sys.stdin.fileno(), termios.TCSADRAIN, old)
        stop.set()


def main():
    # Split commands by ';;' (unlikely to occur in a real cmd line) or
    # newlines. \r, \n, \xHH etc escape sequences are interpreted. A
    # literal CR gets appended to each command if not already present.
    stim_env = os.environ.get("BATOS_HV_STIMULUS", "")
    stims = []
    if stim_env:
        for raw in stim_env.replace("\n", ";;").split(";;"):
            raw = raw.strip()
            if not raw:
                continue
            decoded = raw.encode("utf-8").decode("unicode_escape").encode("latin-1")
            if not decoded.endswith(b"\r") and not decoded.endswith(b"\n"):
                decoded = decoded + b"\r"
            stims.append(decoded)

    # ttyACM2 may lag a few seconds behind ttyACM1 after chainload.
    # Retry up to 30 s before giving up.
    vuart = None
    deadline = time.time() + 30
    while time.time() < deadline:
        try:
            vuart = serial.Serial(
                "/dev/ttyACM2",
                baudrate=115200,
                timeout=0.1,
                rtscts=False, xonxoff=False, dsrdtr=False,
            )
            break
        except (FileNotFoundError, serial.SerialException) as e:
            last_err = e
            time.sleep(1.0)
    if vuart is None:
        log(f"timed out waiting for /dev/ttyACM2: {last_err}")
        sys.exit(1)
    configure_vuart_raw(vuart)

    stop = threading.Event()
    threading.Thread(target=vuart_reader, args=(vuart,), daemon=True).start()
    if stims:
        threading.Thread(target=stimulus_sender,
                         args=(vuart, stims), daemon=True).start()

    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)
    hv = HV(iface, p, u)
    hv.init()

    log(f"loading {BAT_OS_BINARY}")
    hv.load_raw(BAT_OS_BINARY.read_bytes(), 0)

    log("calling hv.start() — Bat_OS takes over now. "
        f"Ctrl+] to detach.")

    # hv.start() must run on the main thread because m1n1 installs
    # a SIGINT handler for the shell. For interactive input we run
    # the stdin forwarder in a worker thread.
    if not stims and sys.stdin.isatty():
        threading.Thread(target=stdin_forwarder,
                         args=(vuart, stop), daemon=True).start()

    try:
        hv.start()
    except KeyboardInterrupt:
        pass
    except Exception as e:
        sys.stderr.write(f"\n[host] hv.start() raised: {e}\n")
        sys.stderr.flush()

    stop.set()
    log("detaching — draining vuart for 2s")
    time.sleep(2)
    vuart.close()


if __name__ == "__main__":
    main()
