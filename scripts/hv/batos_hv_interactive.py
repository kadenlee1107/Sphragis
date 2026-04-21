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
import sys, os, time, termios, tty, threading, pathlib, select, signal

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


def stimulus_sender(vuart, items, verbose):
    """Fire each byte-string in items after a delay.

    BATOS_HV_STIM_GAP_S overrides the between-item sleep (default 0.8 s).
    Useful when the next stim needs to arrive at a later phase (e.g.
    tab-to-X keystrokes must land AFTER boot_screen exits, which can
    take 15+ s).
    """
    time.sleep(1.5)  # give the guest time to print the prompt
    gap = float(os.environ.get("BATOS_HV_STIM_GAP_S", "0.8"))
    for stim in items:
        if verbose:
            sys.stderr.write(f"\n[vuart] >>> {stim!r}\n")
            sys.stderr.flush()
        vuart.write(stim)
        vuart.flush()
        time.sleep(gap)


# Shared state populated by main() once the proxy iface exists.
# vuart_reader watches Bat_OS uart output for the halt marker and,
# when seen, kicks the HV out of hv_start (via `!` byte) and sets
# a halt flag. The wrapped HV.run_shell (installed in main) checks
# that flag and raises ExitConsole(EXC_RET.EXIT_GUEST) on entry
# WITHOUT calling interact() — which would otherwise EOF-exit
# immediately since stdin is /dev/null for backgrounded runs.
# EXIT_GUEST bubbles through handle_exception → p.exit(3) →
# m1n1's hv_exit_guest() → hv.start() returns on Python side →
# Mac is back in stock proxy mode, ready for chainload without a
# physical power-cycle.
_iface_for_kick = {"iface": None}
_halt_seen = threading.Event()
_HALT_MARKER = b"[BATOS] halt requested via UI close button"


def vuart_reader(vuart):
    buf = b""
    kicked = False
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
            if not kicked:
                buf = (buf + d)[-256:]
                if _HALT_MARKER in buf and _iface_for_kick["iface"] is not None:
                    kicked = True
                    sys.stderr.write("\n[host] Bat_OS halt marker — kicking HV for clean exit\n")
                    sys.stderr.flush()
                    # Small delay so Bat_OS has entered its wfe loop
                    # and the halt message is fully flushed.
                    time.sleep(0.3)
                    _halt_seen.set()
                    try:
                        _iface_for_kick["iface"].dev.write(b"!")
                    except Exception as e:
                        sys.stderr.write(f"[host] kick write failed: {e}\n")
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


def _noop_signal(*_):
    # Placeholder handler so a stray SIGUSR2 before run_shell installs
    # its own handler doesn't kill the process with Python's default
    # terminate-on-USR2 behaviour.
    pass


def main():
    # Default USR2 to no-op. run_shell's own handler (which raises
    # ExitConsole(EXC_RET.EXIT_GUEST)) overrides this while it's active.
    signal.signal(signal.SIGUSR2, _noop_signal)

    # Split commands by ';;' (unlikely to occur in a real cmd line) or
    # newlines. \r, \n, \xHH etc escape sequences are interpreted. A
    # literal CR gets appended to each command if not already present
    # UNLESS the stim already contains control characters (tabs,
    # explicit CRs) — in that case treat it as a byte-level stim and
    # leave it alone.
    stim_env = os.environ.get("BATOS_HV_STIMULUS", "")
    stims = []
    if stim_env:
        for raw in stim_env.replace("\n", ";;").split(";;"):
            if not raw:
                continue
            decoded = raw.encode("utf-8").decode("unicode_escape").encode("latin-1")
            # Byte-level stim — has tabs or explicit CRs → don't touch
            has_control = any(b in (0x09, 0x0d, 0x1b) for b in decoded)
            if not has_control:
                decoded = decoded.strip()
                if not decoded:
                    continue
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
    verbose = os.environ.get("BATOS_HV_VERBOSE", "1") != "0"
    if stims:
        threading.Thread(target=stimulus_sender,
                         args=(vuart, stims, verbose), daemon=True).start()

    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)
    hv = HV(iface, p, u)
    # Make iface available to vuart_reader so it can kick HV on halt marker.
    _iface_for_kick["iface"] = iface
    # Wrap hv.run_shell so when the halt flag is set we exit the guest
    # WITHOUT entering the interactive shell (which would EOF-exit on
    # /dev/null stdin and return None → HANDLED instead of EXIT_GUEST).
    # Returning EXIT_GUEST here propagates through handle_exception →
    # p.exit(3) → m1n1's EXC_EXIT_GUEST → hv_exit_guest() → hv.start
    # returns on the Python side.
    from m1n1.proxy import EXC_RET as _EXC_RET
    _orig_run_shell = hv.run_shell
    def _halt_aware_run_shell(entry_msg="Entering shell", exit_msg="Continuing"):
        if _halt_seen.is_set():
            sys.stderr.write("[host] run_shell intercepted — halt flag set, "
                             "returning EXIT_GUEST\n")
            sys.stderr.flush()
            return _EXC_RET.EXIT_GUEST
        return _orig_run_shell(entry_msg, exit_msg)
    hv.run_shell = _halt_aware_run_shell
    hv.init()

    # Allow overriding payload for diagnostic experiments (e.g. a
    # wfi-forever stub to isolate whether guest activity matters).
    payload_path = os.environ.get("BATOS_HV_PAYLOAD", str(BAT_OS_BINARY))
    log(f"loading {payload_path}")
    hv.load_raw(pathlib.Path(payload_path).read_bytes(), 0)

    # Optional pre-start wall-clock delay for watchdog-source experiments.
    # If the Mac's 113 s ceiling is measured from iBoot handoff, delaying
    # hv.start() by N seconds will reduce HV runtime by N. If it's
    # measured from hv.start(), the HV portion stays ~113 s regardless.
    prestart_s = int(os.environ.get("BATOS_HV_PRESTART_SLEEP", "0"))
    if prestart_s > 0:
        log(f"BATOS_HV_PRESTART_SLEEP={prestart_s}s — waiting before hv.start()")
        time.sleep(prestart_s)

    # Optional "init only" — call hv.init() but never hv.start(). Used
    # to test whether the M4 reset-watchdog is armed by hv_init itself
    # or requires hv_start / live guest execution.
    if os.environ.get("BATOS_HV_INIT_ONLY", "0") == "1":
        log("BATOS_HV_INIT_ONLY=1 — not calling hv.start(); sleeping 200s")
        for i in range(200):
            if i % 10 == 0:
                log(f"init-only t={i}s")
            time.sleep(1)
        log("init-only done")
        stop.set()
        vuart.close()
        return

    log("calling hv.start() — Bat_OS takes over now. "
        f"Ctrl+] to detach.")

    # hv.start() must run on the main thread because m1n1 installs
    # a SIGINT handler for the shell. For interactive input we run
    # the stdin forwarder in a worker thread.
    #
    # 2026-04-20 20:30: used to be gated `if not stims` — meaning when
    # a canned stimulus was provided, stdin was ignored. That's wrong:
    # we want the canned stimulus to fire ONCE at start (for auto-auth
    # etc.) AND leave stdin active so the operator can type keys
    # (Tab, Enter, etc.) after the guest is up. So always forward
    # stdin when it's a tty — the stim thread fires its items then
    # exits, stdin keeps flowing.
    if sys.stdin.isatty():
        threading.Thread(target=stdin_forwarder,
                         args=(vuart, stop), daemon=True).start()

    try:
        hv.start()
        # Reached if the last guest CPU exits cleanly (Bat_OS's
        # halt_bat_os writes CYC_OVRD_EL1 bit 0 → m1n1 HV sees "Guest
        # is shutting down CPU" → hv_exit_cpu → hv_start returns).
        # Mac is now in stock "Running proxy..." state; chainload.py
        # works without a power-cycle.
        log("hv.start() returned cleanly — Mac is back in m1n1 proxy "
            "mode. You can re-chainload without a power-cycle.")
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
