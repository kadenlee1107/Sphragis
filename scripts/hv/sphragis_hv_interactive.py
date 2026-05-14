#!/usr/bin/env python3
"""Run Sphragis under m1n1's hypervisor and drive an interactive shell
session over /dev/ttyACM2 (the USB-CDC vuart secondary endpoint).

This script combines m1n1's run_guest.py functionality with a
pyserial-backed reader/writer thread on the vuart endpoint, so the
host tty driver can't drop the connection between reads and the
Sphragis guest can receive bytes we type.

Prerequisites:
  * Patched m1n1.macho already chainloaded on the Mac (see
    `docs/SESSION_JOURNAL.md` for the chainload command).
  * udev rule granting dialout group access to m1n1 uartproxy.
  * User in group dialout (or wrap with `sg dialout -c '...'`).

Usage:
  # Interactive REPL — stdin bytes go to the guest, guest TX goes
  # to stdout. Ctrl+] to detach cleanly.
  sg dialout -c "/usr/bin/python3 scripts/hv/sphragis_hv_interactive.py"

  # Canned stimulus — just fire one command and print the response.
  # CR is appended automatically if missing; ';;' separates commands.
  SPHRAGIS_HV_STIMULUS="help" sg dialout -c \\
      "/usr/bin/python3 scripts/hv/sphragis_hv_interactive.py"

  # Multiple stimuli (separated by ';;' or newlines) with 500 ms gaps:
  SPHRAGIS_HV_STIMULUS="uname;;mem;;uptime" sg dialout -c ...

  # Infinite demo reel — one invocation, N back-to-back Sphragis sessions
  # with fresh m1n1 chainloaded in-process between each. Ctrl+C to stop.
  SPHRAGIS_HV_LOOP=1 SPHRAGIS_HV_STIMULUS=$'batman;;\t\t\t\t\t\t\t\t\t\r' \\
    SPHRAGIS_HV_STIM_GAP_S=25 sg dialout -c \\
      "/usr/bin/python3 scripts/hv/sphragis_hv_interactive.py"

  # Same, but stop after 3 iterations (smoke-test variant):
  SPHRAGIS_HV_LOOP=1 SPHRAGIS_HV_LOOP_MAX=3 sg dialout -c ...

Output:
  * Bytes from the guest (ttyACM2 vuart) print to stdout live.
  * Bytes from the m1n1 proxy (ttyACM1) print prefixed with
    `TTY>` — that's m1n1's own library formatting.
  * Our host-side status lines are prefixed `[host]` / `[vuart]`
    and go to stderr so they don't pollute captured logs.
"""
import sys
import os
import time
import termios
import tty
import threading
import pathlib
import select
import signal

M1N1_ROOT = pathlib.Path(__file__).resolve().parents[2] / "external/m1n1"
sys.path.insert(0, str(M1N1_ROOT / "proxyclient"))

import serial
from m1n1.proxy import *
from m1n1.proxyutils import *
from m1n1.utils import *
from m1n1.hv import HV

SPHRAGIS_BINARY = pathlib.Path(__file__).resolve().parents[2] / \
    "target/sphragis_apple.bin"

ESCAPE_BYTE = 0x1d   # Ctrl+] — same as miniterm convention.


def log(msg):
    sys.stderr.write(f"[host] {msg}\n")
    sys.stderr.flush()


def configure_vuart_raw(vuart):
    """Raw termios + DTR asserted. Without this, Ubuntu's tty layer
    echo+canonical settings create an infinite CRLF echo loop that
    looks exactly like Sphragis is broken.

    Also clears HUPCL so the kernel DOESN'T drop DTR when the last
    fd closes. On USB-CDC the Mac's m1n1 interprets DTR-toggle as
    "host disconnected" and wedges its USB stack — any subsequent
    pyserial.Serial() open against /dev/ttyACM* then blocks forever
    even though the device node still exists. Clearing HUPCL keeps
    modem-control lines steady across our process lifetime.

    DTR/RTS ioctls are best-effort: some m1n1 USB CDC states return
    EPROTO on TIOCMBIC (errno 71). That's non-fatal for us — we
    only need the data pipe, and HUPCL clearing below is the real
    DTR stabiliser."""
    try:
        vuart.dtr = True
    except OSError as e:
        log(f"vuart.dtr=True failed (non-fatal): {e!r}")
    try:
        vuart.rts = False
    except OSError as e:
        log(f"vuart.rts=False failed (non-fatal): {e!r}")
    _clear_hupcl_and_set_raw(vuart.fileno())


def _clear_hupcl_and_set_raw(fd):
    a = termios.tcgetattr(fd)
    a[0] &= ~(termios.BRKINT | termios.ICRNL | termios.INPCK
              | termios.ISTRIP | termios.IXON)
    a[1] &= ~termios.OPOST
    a[2] = (a[2] & ~(termios.CSIZE | termios.PARENB | termios.HUPCL)) | termios.CS8
    a[3] &= ~(termios.ECHO | termios.ECHONL | termios.ICANON
              | termios.ISIG | termios.IEXTEN)
    termios.tcsetattr(fd, termios.TCSANOW, a)


def stimulus_sender(ref, items, verbose):
    """Fire each byte-string in items after a delay.

    SPHRAGIS_HV_STIM_GAP_S overrides the between-item sleep (default 0.8 s).
    Useful when the next stim needs to arrive at a later phase (e.g.
    tab-to-X keystrokes must land AFTER boot_screen exits, which can
    take 15+ s).

    Reads the vuart from ref each write so that after an in-loop
    chainload swaps USB devices, the retry loop picks up the new fd.
    Writes are retried on EIO / SerialException: m1n1's
    hv_map_vuart_dockchannel briefly transitions USB_VUART between
    console iodev and dockchannel mapping during hv.start(), and a
    write landing in that window returns errno 5.
    """
    time.sleep(1.5)  # give the guest time to print the prompt
    gap = float(os.environ.get("SPHRAGIS_HV_STIM_GAP_S", "0.8"))
    for stim in items:
        if verbose:
            sys.stderr.write(f"\n[vuart] >>> {stim!r}\n")
            sys.stderr.flush()
        _vuart_write_with_retry(ref, stim)
        time.sleep(gap)


def _vuart_write_with_retry(ref, data, attempts=20, delay=0.25):
    for i in range(attempts):
        vuart = ref["vuart"]
        if vuart is None:
            time.sleep(delay)
            continue
        try:
            vuart.write(data)
            vuart.flush()
            return
        except (OSError, serial.SerialException) as e:
            if i == attempts - 1:
                sys.stderr.write(
                    f"[vuart] write gave up after {attempts} retries: {e!r}\n")
                sys.stderr.flush()
                return
            time.sleep(delay)


# Shared state populated by main() once the proxy iface exists.
# vuart_reader watches Sphragis uart output for the halt marker and,
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
_HALT_MARKER = b"[SPHRAGIS] halt requested via UI close button"
# Mutable vuart reference so the long-running reader can pick up the
# new fd after each in-loop chainload_inline re-enumerates USB. main()
# writes here every time it (re)opens a vuart.
_vuart_ref = {"vuart": None}


def vuart_reader(ref):
    # _halt_seen is the armed/disarmed gate. main() clears it between
    # iterations when SPHRAGIS_HV_LOOP=1 to re-arm halt detection for the
    # next Sphragis session.
    #
    # Transient read errors (SerialException / OSError) happen during
    # m1n1's hv_map_vuart_dockchannel remap and after chainload_inline
    # swaps the underlying USB device — returning here would end the
    # thread permanently and kill halt detection. Instead, sleep
    # briefly and retry against whatever vuart ref points at now.
    buf = b""
    while True:
        vuart = ref["vuart"]
        if vuart is None:
            time.sleep(0.1)
            continue
        try:
            d = vuart.read(1024)
        except (serial.SerialException, OSError) as e:
            sys.stderr.write(f"\n[vuart] transient read error: {e!r}\n")
            sys.stderr.flush()
            time.sleep(0.25)
            continue
        if d:
            sys.stdout.buffer.write(d)
            sys.stdout.buffer.flush()
            buf = (buf + d)[-256:]
            if (_HALT_MARKER in buf
                    and not _halt_seen.is_set()
                    and _iface_for_kick["iface"] is not None):
                sys.stderr.write("\n[host] Sphragis halt marker — kicking HV for clean exit\n")
                sys.stderr.flush()
                # Small delay so Sphragis has entered its wfe loop
                # and the halt message is fully flushed.
                time.sleep(0.3)
                _halt_seen.set()
                try:
                    _iface_for_kick["iface"].dev.write(b"!")
                except Exception as e:
                    sys.stderr.write(f"[host] kick write failed: {e}\n")
                # Drop the marker so that when main clears _halt_seen
                # for the next iteration, stale bytes don't re-fire.
                buf = b""
        time.sleep(0.005)


def stdin_forwarder(ref, stop):
    """If stdin is a tty, put it in cbreak mode and forward each key
    to the guest. Ctrl+] detaches. Reads the vuart from ref each
    write so that inter-iteration chainloads don't kill typing."""
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
                vuart = ref["vuart"]
                if vuart is None:
                    continue
                try:
                    vuart.write(b)
                    vuart.flush()
                except Exception as e:
                    sys.stderr.write(f"[vuart] write failed: {e}\n")
                    # Don't break — ref may swap to a fresh vuart.
    finally:
        termios.tcsetattr(sys.stdin.fileno(), termios.TCSADRAIN, old)
        stop.set()


def _noop_signal(*_):
    # Placeholder handler so a stray SIGUSR2 before run_shell installs
    # its own handler doesn't kill the process with Python's default
    # terminate-on-USR2 behaviour.
    pass


# HID usage ID → ASCII byte. From USB HID Usage Tables, Keyboard Page.
# Shift column = what the key produces with Shift held.
# 0-255 are raw integers; 'a' etc are strings (encoded later).
_HID_TO_ASCII = {
    # Letters
    0x04: ('a', 'A'), 0x05: ('b', 'B'), 0x06: ('c', 'C'), 0x07: ('d', 'D'),
    0x08: ('e', 'E'), 0x09: ('f', 'F'), 0x0a: ('g', 'G'), 0x0b: ('h', 'H'),
    0x0c: ('i', 'I'), 0x0d: ('j', 'J'), 0x0e: ('k', 'K'), 0x0f: ('l', 'L'),
    0x10: ('m', 'M'), 0x11: ('n', 'N'), 0x12: ('o', 'O'), 0x13: ('p', 'P'),
    0x14: ('q', 'Q'), 0x15: ('r', 'R'), 0x16: ('s', 'S'), 0x17: ('t', 'T'),
    0x18: ('u', 'U'), 0x19: ('v', 'V'), 0x1a: ('w', 'W'), 0x1b: ('x', 'X'),
    0x1c: ('y', 'Y'), 0x1d: ('z', 'Z'),
    # Numbers / top row
    0x1e: ('1', '!'), 0x1f: ('2', '@'), 0x20: ('3', '#'), 0x21: ('4', '$'),
    0x22: ('5', '%'), 0x23: ('6', '^'), 0x24: ('7', '&'), 0x25: ('8', '*'),
    0x26: ('9', '('), 0x27: ('0', ')'),
    # Named
    0x28: ('\r', '\r'),   # Enter
    0x29: ('\x1b', '\x1b'),  # Escape
    0x2a: ('\x7f', '\x7f'),  # Backspace (DEL to be consistent with Linux)
    0x2b: ('\t', '\t'),   # Tab
    0x2c: (' ', ' '),     # Space
    # Punctuation
    0x2d: ('-', '_'), 0x2e: ('=', '+'),
    0x2f: ('[', '{'), 0x30: (']', '}'), 0x31: ('\\', '|'),
    0x33: (';', ':'), 0x34: ("'", '"'), 0x35: ('`', '~'),
    0x36: (',', '<'), 0x37: ('.', '>'), 0x38: ('/', '?'),
    # Arrows — emit ANSI escape sequences for shell compatibility
    0x4f: ('\x1b[C', '\x1b[C'),   # Right
    0x50: ('\x1b[D', '\x1b[D'),   # Left
    0x51: ('\x1b[B', '\x1b[B'),   # Down
    0x52: ('\x1b[A', '\x1b[A'),   # Up
}


def _hid_decode(mods, key):
    """Return bytes for one HID key event, given modifier byte and key code.
    Returns empty bytes for unmapped keys."""
    entry = _HID_TO_ASCII.get(key)
    if entry is None:
        return b""
    shift = bool(mods & 0x22)  # L Shift | R Shift
    ctrl  = bool(mods & 0x11)  # L Ctrl | R Ctrl
    c = entry[1 if shift else 0]
    if ctrl and len(c) == 1 and 'a' <= c.lower() <= 'z':
        # Ctrl+letter → ASCII 0x01..0x1A
        return bytes([ord(c.lower()) - ord('a') + 1])
    return c.encode("latin-1")


def _mtp_kbd_probe(iface, p, u, vuart):
    """Set up SMC + DART-MTP + dockchannel-mtp + MTP ASC, subscribe
    to the keyboard MTP interface, poll forever. Each key event
    prints to stderr and, if SPHRAGIS_HV_MTP_BRIDGE_TO_VUART=1, writes
    the decoded byte(s) through the vuart so Sphragis sees them via
    platform::serial_getc same as the Linux typing path."""
    log("MTP keyboard probe starting — importing m1n1 fw modules")
    from m1n1.fw.asc import StandardASC
    from m1n1.hw.dart import DART
    from m1n1.hw.dockchannel import DockChannel
    from m1n1.fw.smc import SMCClient
    from m1n1.fw.mtp import (
        MTPProtocol, MTPKeyboardInterface,
    )

    bridge = os.environ.get("SPHRAGIS_HV_MTP_BRIDGE_TO_VUART", "0") == "1"
    log(f"bridge-to-vuart: {bridge}")

    smc_addr = u.adt["arm-io/smc"].get_reg(0)[0]
    log(f"SMC @ 0x{smc_addr:x}")
    smc = SMCClient(u, smc_addr)
    smc.start()
    smc.start_ep(0x20)
    smc.verbose = 0

    # M4 experiment: dapf_init_all hangs partway through on M4 (dart-aop
    # inits ok, dart-mtp write then never acks). Bypass DAPF entirely
    # and set BYPASS_DAPF=1 in the DART TCR below — DAPF is a security
    # filter; without it the DART accepts all transactions, which is
    # fine for a bring-up keyboard probe.
    skip_dapf = os.environ.get("SPHRAGIS_HV_MTP_SKIP_DAPF", "1") == "1"
    if not skip_dapf:
        _saved_timeout = iface.dev.timeout
        iface.dev.timeout = 60
        log("p.dapf_init_all() (timeout bumped to 60s)...")
        try:
            p.dapf_init_all()
        finally:
            iface.dev.timeout = _saved_timeout
    else:
        log("skipping dapf_init_all (SPHRAGIS_HV_MTP_SKIP_DAPF=1)")

    log("DART /arm-io/dart-mtp...")
    dart = DART.from_adt(u, "/arm-io/dart-mtp", iova_range=(0x8000, 0x100000))
    dart.dart.regs.TCR[1].set(BYPASS_DAPF=int(skip_dapf), BYPASS_DART=0,
                              TRANSLATE_ENABLE=1)

    irq_base = u.adt["/arm-io/dockchannel-mtp"].get_reg(1)[0]
    fifo_base = u.adt["/arm-io/dockchannel-mtp"].get_reg(2)[0]
    log(f"dockchannel-mtp irq=0x{irq_base:x} fifo=0x{fifo_base:x}")
    dc = DockChannel(u, irq_base, fifo_base, 1)

    node = u.adt["/arm-io/dockchannel-mtp/mtp-transport"]

    # Drain stale bytes
    while dc.rx_count:
        dc.read(dc.rx_count)

    mtp_addr = u.adt["/arm-io/mtp"].get_reg(0)[0]
    log(f"MTP ASC @ 0x{mtp_addr:x}")
    mtp = StandardASC(u, mtp_addr, dart, stream=1)
    mtp.verbose = 1
    mtp.allow_phys = True

    # Inspect ASC CPU_CONTROL state BEFORE trying anything — tells us
    # whether macOS/iBoot left the coprocessor running.
    try:
        cpu_ctrl = p.read32(mtp_addr + 0x0044)
        cpu_status = p.read32(mtp_addr + 0x0048)
        log(f"MTP ASC pre-boot: CPU_CONTROL=0x{cpu_ctrl:08x} "
            f"CPU_STATUS=0x{cpu_status:08x}")
    except Exception as e:
        log(f"couldn't read ASC regs: {e!r}")

    from m1n1.fw.asc.base import ASCTimeout

    mode = os.environ.get("SPHRAGIS_HV_MTP_BOOT_MODE", "boot")
    # Valid modes:
    #   "boot"        — call mtp.boot() with default timeout
    #   "boot-long"   — boot with 10s wait
    #   "stop-boot"   — stop() then boot()
    #   "skip"        — assume already running, attach to mailbox
    #   "cascade"     — try boot-long, then stop-boot, then skip
    log(f"mtp boot mode: {mode}")

    def try_boot_long():
        # StandardASC.boot wraps mgmt.wait_boot(1); reach in with longer wait.
        mtp.boot_long_ran = True
        # Just run the parts of boot() with longer wait_boot
        print("Booting ASC (long timeout)...")
        mtp.asc.CPU_CONTROL.set(RUN=1)
        mtp.mgmt.wait_boot(10)

    def try_stop_boot():
        log("stop+boot dance")
        try:
            mtp.stop()
        except Exception as e:
            log(f"  stop failed (expected if not yet booted): {e!r}")
        mtp.boot()

    if mode == "boot":
        mtp.boot()
    elif mode == "boot-long":
        try_boot_long()
    elif mode == "stop-boot":
        try_stop_boot()
    elif mode == "skip":
        log("skipping mtp.boot() — assuming ASC already running")
    elif mode == "cascade":
        try:
            log("attempt 1: boot-long")
            try_boot_long()
            log("  OK")
        except ASCTimeout:
            log("  timeout — attempt 2: stop+boot")
            try:
                try_stop_boot()
                log("  OK")
            except Exception as e:
                log(f"  failed: {e!r} — attempt 3: skip")
                # leave ASC alone; go straight to MTPProtocol setup
    else:
        raise RuntimeError(f"unknown SPHRAGIS_HV_MTP_BOOT_MODE={mode}")

    # Subclass MTPKeyboardInterface to capture + decode HID reports.
    class BridgeKeyboard(MTPKeyboardInterface):
        NAME = "keyboard"

        def __init__(self, *a, **kw):
            super().__init__(*a, **kw)
            self.last_keys = set()

        def report(self, msg):
            # msg is the raw HID report payload (bytes-like).
            data = bytes(msg)
            sys.stderr.write(f"[mtp-kbd] HID report: {data.hex()}\n")
            sys.stderr.flush()
            if len(data) < 3:
                return
            mods = data[0]
            # byte 1 is reserved; bytes 2..7 are simultaneously-pressed keys
            keys = set(k for k in data[2:8] if k != 0)
            newly_pressed = keys - self.last_keys
            self.last_keys = keys
            for k in newly_pressed:
                out = _hid_decode(mods, k)
                if out:
                    sys.stderr.write(f"[mtp-kbd] key 0x{k:02x} mods=0x{mods:02x} -> {out!r}\n")
                    sys.stderr.flush()
                    if bridge:
                        try:
                            vuart.write(out)
                            vuart.flush()
                        except Exception as e:
                            sys.stderr.write(f"[mtp-kbd] vuart write err: {e!r}\n")

    # Patch the keyboard class used by the protocol.
    # MTPProtocol looks up interfaces by NAME in its INTERFACES list
    # when init messages arrive — swap in our subclass.
    from m1n1.fw import mtp as _mtp_mod
    for i, cls in enumerate(_mtp_mod.MTPProtocol.INTERFACES):
        if cls.NAME == "keyboard":
            _mtp_mod.MTPProtocol.INTERFACES[i] = BridgeKeyboard

    log("creating MTPProtocol, waiting for keyboard init...")
    mp = MTPProtocol(u, node, mtp, dc, smc)
    try:
        mp.wait_init("keyboard")
    except Exception as e:
        log(f"wait_init failed: {e!r}")
        raise

    mtp.stop()
    mtp.start()
    log("keyboard initialized — polling for key events. Type on the Mac keyboard!")
    log("Ctrl+C / kill -9 to exit.")

    try:
        while True:
            try:
                mp.work_pending()
                mtp.work()
            except Exception as e:
                log(f"work error: {e!r}")
                time.sleep(0.1)
            time.sleep(0.005)
    except KeyboardInterrupt:
        pass


def _dump_keyboard_adt(adt):
    """Walk the ADT looking for keyboard/HID-transport nodes and
    their containing SPI controllers. Writes an annotated listing
    to stderr + /tmp/adt_kbd.log so we can pick the right MMIO
    base for Sphragis's SPI keyboard driver.

    Looks for:
      - `spi-1,spimc` (M4-class SPI controller)
      - any child with compatible containing `hid` or `keyboard`
      - `hid-transport,spi` (older MBP SPI keyboard)
      - `k1`-style HID keyboards (M1/M2)
      - any direct `keyboard` / `kbd` node names
    """
    import io
    out = io.StringIO()

    def pline(s):
        sys.stderr.write(s + "\n")
        out.write(s + "\n")

    pline("=" * 70)
    pline("[adt-kbd] keyboard + HID-transport + SPI controller scan")
    pline("=" * 70)

    spi_controllers = []
    hid_nodes = []
    name_hits = []

    for node in adt.walk_tree():
        try:
            compat = getattr(node, "compatible", None)
            if isinstance(compat, (list, tuple)) and len(compat) > 0:
                c0 = compat[0]
                if "spimc" in c0 or c0.startswith("spi"):
                    spi_controllers.append((node, compat))
                if "hid" in c0.lower() or "keyboard" in c0.lower():
                    hid_nodes.append((node, compat))
        except Exception:
            pass
        try:
            nm = getattr(node, "name", "")
            if nm and ("kbd" in nm.lower() or "keyboard" in nm.lower() or
                      "hid" in nm.lower()):
                name_hits.append(node)
        except Exception:
            pass

    pline(f"[adt-kbd] SPI controllers found ({len(spi_controllers)}):")
    for node, compat in spi_controllers:
        try:
            pline(f"  {node.name}  compatible={list(compat)}")
            # try to get reg / base address
            try:
                reg = node.get_reg(0)
                pline(f"    reg[0] = {reg}")
            except Exception as e:
                pline(f"    reg[0]: <{e!r}>")
            # list its children
            for c in node:
                try:
                    cc = list(getattr(c, "compatible", []))
                    pline(f"    child: {c.name}  compatible={cc}")
                except Exception:
                    pline(f"    child: {c.name}  (no compat)")
        except Exception as e:
            pline(f"  <walk err: {e!r}>")

    pline(f"[adt-kbd] HID/keyboard compatibles ({len(hid_nodes)}):")
    for node, compat in hid_nodes:
        try:
            parent = getattr(node, "_parent", None)
            ppath = parent.name if parent else "?"
            pline(f"  {ppath}/{node.name}  compatible={list(compat)}")
        except Exception as e:
            pline(f"  <walk err: {e!r}>")

    pline(f"[adt-kbd] name-based hits ({len(name_hits)}):")
    for node in name_hits:
        try:
            parent = getattr(node, "_parent", None)
            ppath = parent.name if parent else "?"
            try:
                compat = list(getattr(node, "compatible", []))
            except Exception:
                compat = []
            pline(f"  {ppath}/{node.name}  compatible={compat}")
        except Exception as e:
            pline(f"  <walk err: {e!r}>")

    # M4/M3 keyboard path goes through AOP (Always-On Processor) +
    # MTP (MultiTouch Protocol). Dump every node whose name or
    # compat contains mtp/aop/dart/dockchannel/smc so we can see
    # M4's exact coprocessor topology.
    pline("")
    pline("[adt-kbd] MTP/AOP/DART/Dockchannel/SMC topology (M4 keyboard path):")
    wanted = ("mtp", "aop", "dart", "dockchannel", "smc", "asc", "rtkit")
    for node in adt.walk_tree():
        try:
            nm = getattr(node, "name", "")
            compat = []
            try:
                compat = list(getattr(node, "compatible", []))
            except Exception:
                pass
            nm_l = nm.lower()
            c_join = " ".join(compat).lower()
            if any(w in nm_l for w in wanted) or any(w in c_join for w in wanted):
                try:
                    parent = getattr(node, "_parent", None)
                    ppath = parent.name if parent else "?"
                except Exception:
                    ppath = "?"
                pline(f"  {ppath}/{nm}  compatible={compat}")
                # All reg ranges
                try:
                    # ADT Node objects don't have a uniform get_reg — try
                    # the property accessors the fw/mtp path uses.
                    for regi in range(6):
                        try:
                            r = node.get_reg(regi)
                            if r is None:
                                break
                            pline(f"    reg[{regi}] = (0x{r[0]:x}, 0x{r[1]:x})")
                        except Exception:
                            break
                except Exception:
                    pass
                try:
                    irqs = getattr(node, "interrupts", None)
                    if irqs:
                        pline(f"    interrupts = {list(irqs)}")
                except Exception:
                    pass
                # Also list children
                try:
                    children = []
                    for c in node:
                        try:
                            cc = list(getattr(c, "compatible", []))
                        except Exception:
                            cc = []
                        children.append(f"{c.name}{cc}")
                    if children:
                        pline(f"    children: {children}")
                except Exception:
                    pass
                # Interesting properties
                try:
                    props = [k for k in dir(node) if not k.startswith("_")
                             and not callable(getattr(node, k, None))]
                    interesting = [k for k in props if any(
                        s in k.lower() for s in
                        ["endpoint", "rtk", "mbox", "asc", "iommu",
                         "function", "ep-", "ep_", "mailbox"])]
                    if interesting:
                        pline(f"    props-of-interest: {interesting[:16]}")
                except Exception:
                    pass
        except Exception:
            pass

    pline("=" * 70)
    try:
        with open("/tmp/adt_kbd.log", "w") as f:
            f.write(out.getvalue())
        sys.stderr.write("[adt-kbd] also written to /tmp/adt_kbd.log\n")
    except Exception as e:
        sys.stderr.write(f"[adt-kbd] write /tmp/adt_kbd.log failed: {e!r}\n")


def _build_hv(iface, p, heap_size=128 * 1024 * 1024):
    """Build a fresh ProxyUtils + HV against the proxy's current m1n1
    session, with run_shell wrapped to return EXIT_GUEST whenever
    _halt_seen is set. Called once at startup and again after each
    chainload_inline() when SPHRAGIS_HV_LOOP=1 re-arms for another
    Sphragis demo cycle."""
    from m1n1.proxy import EXC_RET as _EXC_RET
    u = ProxyUtils(p, heap_size=heap_size)
    hv = HV(iface, p, u)
    _orig_run_shell = hv.run_shell

    def _halt_aware_run_shell(entry_msg="Entering shell",
                              exit_msg="Continuing"):
        if _halt_seen.is_set():
            sys.stderr.write(
                "[host] run_shell intercepted — halt flag set, "
                "returning EXIT_GUEST\n")
            sys.stderr.flush()
            return _EXC_RET.EXIT_GUEST
        return _orig_run_shell(entry_msg, exit_msg)

    hv.run_shell = _halt_aware_run_shell
    return u, hv


def _post_exit_diag(p, iteration):
    """Sanity-ping the proxy and rewire USB_VUART after hv.start()
    returns cleanly. Same probes used to close out the no-power-cycle
    investigation — kept here because they're cheap signal on whether
    the fresh m1n1 is still talking before we re-chainload."""
    try:
        p.nop()
        log(f"[iter {iteration}] post-exit probe: p.nop() OK")
    except Exception as e:
        log(f"[iter {iteration}] post-exit probe: p.nop() FAIL: {e!r}")

    try:
        base = p.get_base()
        log(f"[iter {iteration}] post-exit probe: p.get_base()=0x{base:x}")
    except Exception as e:
        log(f"[iter {iteration}] post-exit probe: p.get_base() FAIL: {e!r}")

    try:
        from m1n1.proxy import IODEV, USAGE
        p.iodev_set_usage(IODEV.USB_VUART,
                          USAGE.CONSOLE | USAGE.UARTPROXY)
        log(f"[iter {iteration}] post-exit probe: iodev_set_usage("
            f"USB_VUART, CONSOLE|UARTPROXY) OK")
    except Exception as e:
        log(f"[iter {iteration}] post-exit probe: iodev_set_usage "
            f"FAIL: {e!r}")

    if os.environ.get("SPHRAGIS_HV_POST_EXIT_FB_SHUTDOWN", "0") == "1":
        try:
            p.fb_shutdown(True)
            log(f"[iter {iteration}] post-exit probe: p.fb_shutdown() OK")
        except Exception as e:
            log(f"[iter {iteration}] post-exit probe: p.fb_shutdown() "
                f"FAIL: {e!r}")


def chainload_inline(iface, p, u, macho_path):
    """Chainload a fresh m1n1 using the existing (iface, p, u) proxy
    session. Ports the logic of external/m1n1/proxyclient/tools/
    chainload.py -S into a callable, so we don't have to open a
    second pyserial fd on /dev/ttyACM1 — which blocks at the
    CDC-ACM driver level while ours is alive, and wedges the Mac's
    USB stack when ours closes (DTR drop, even with HUPCL cleared).

    Always skips secondary CPU RVBAR writes (M4 / t8132 P-cluster
    SErrors on the RVBAR writes, per vendored chainload.py -S).
    """
    from m1n1.macho import MachO
    from m1n1.tgtypes import BootArgs_r1, BootArgs_r2, BootArgs_r3
    from m1n1 import asm
    from m1n1.utils import align

    new_base = u.base
    data = pathlib.Path(macho_path).read_bytes()
    macho = MachO(data)
    image = macho.prepare_image() + b"\x00\x00\x00\x00"
    entry = macho.entry - macho.vmin + new_base

    sepfw_start, sepfw_length = u.adt["chosen"]["memory-map"].SEPFW
    preoslog_start, preoslog_size = 0, 0
    if hasattr(u.adt["chosen"]["memory-map"], "preoslog"):
        preoslog_start, preoslog_size = u.adt["chosen"]["memory-map"].preoslog

    image_size = align(len(image))
    sepfw_off = image_size
    image_size += align(sepfw_length)
    preoslog_off = image_size
    image_size += align(preoslog_size)
    bootargs_off = image_size
    bootargs_size = 0x4000
    image_size += bootargs_size

    print(f"[chainload-inline] total region size 0x{image_size:x}")
    image_addr = u.malloc(image_size)

    print(f"[chainload-inline] loading kernel image (0x{len(image):x} bytes)...")
    u.compressed_writemem(image_addr, image, True)
    p.dc_cvau(image_addr, len(image))

    print(f"[chainload-inline] copying SEPFW (0x{sepfw_length:x} bytes)...")
    p.memcpy8(image_addr + sepfw_off, sepfw_start, sepfw_length)
    u.adt["chosen"]["memory-map"].SEPFW = (new_base + sepfw_off, sepfw_length)
    u.adt["chosen"]["memory-map"].BootArgs = (new_base + bootargs_off, bootargs_size)
    if preoslog_size:
        p.memcpy8(image_addr + preoslog_off, preoslog_start, preoslog_size)
        u.adt["chosen"]["memory-map"].preoslog = (new_base + preoslog_off, preoslog_size)

    print("[chainload-inline] skipping secondary CPU RVBARs (M4 workaround)")
    u.push_adt()

    tba = u.ba.copy()
    tba.top_of_kernel_data = new_base + image_size

    if tba.revision <= 1:
        iface.writemem(image_addr + bootargs_off, BootArgs_r1.build(tba))
    elif tba.revision == 2:
        iface.writemem(image_addr + bootargs_off, BootArgs_r2.build(tba))
    else:
        iface.writemem(image_addr + bootargs_off, BootArgs_r3.build(tba))

    stub = asm.ARMAsm(f"""
1:
        ldp x4, x5, [x1], #16
        stp x4, x5, [x2]
        dc cvau, x2
        ic ivau, x2
        add x2, x2, #16
        sub x3, x3, #16
        cbnz x3, 1b

        ldr x1, ={entry}
        br x1
""", image_addr + image_size)

    iface.writemem(stub.addr, stub.data)
    p.dc_cvau(stub.addr, stub.len)
    p.ic_ivau(stub.addr, stub.len)

    print(f"[chainload-inline] entry=0x{entry:x}")
    print(f"[chainload-inline] reloading into stub at 0x{stub.addr:x}")
    p.reload(stub.addr, new_base + bootargs_off, image_addr, new_base, image_size)

    iface.nop()
    print("[chainload-inline] proxy is alive on new m1n1")


def main():
    # Default USR2 to no-op. run_shell's own handler (which raises
    # ExitConsole(EXC_RET.EXIT_GUEST)) overrides this while it's active.
    signal.signal(signal.SIGUSR2, _noop_signal)

    # SIGTERM handler — if timeout(1) or systemd kills us while hv.start()
    # is blocking in the guest, we'd leave m1n1's HV stuck (no Python
    # left to process USER_INTERRUPT, no clean exit → proxy dies →
    # power-cycle needed). Send the halt kick before exiting so m1n1
    # unwinds the HV cleanly and stays ready for the next launch.
    def _graceful_term(signum, _frame):
        sys.stderr.write(f"\n[host] received signal {signum} — kicking HV\n")
        sys.stderr.flush()
        try:
            if _iface_for_kick["iface"] is not None:
                _halt_seen.set()
                _iface_for_kick["iface"].dev.write(b"!")
        except Exception as e:
            sys.stderr.write(f"[host] graceful kick failed: {e!r}\n")
        # Give m1n1 a moment to unwind before the default SIGTERM lands.
        time.sleep(0.5)
        sys.exit(0)

    signal.signal(signal.SIGTERM, _graceful_term)

    # Split commands by ';;' (unlikely to occur in a real cmd line) or
    # newlines. \r, \n, \xHH etc escape sequences are interpreted. A
    # literal CR gets appended to each command if not already present
    # UNLESS the stim already contains control characters (tabs,
    # explicit CRs) — in that case treat it as a byte-level stim and
    # leave it alone.
    stim_env = os.environ.get("SPHRAGIS_HV_STIMULUS", "")
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

    # Open-vuart is deferred until AFTER the optional bootstrap
    # chainload. Chainload causes m1n1 to re-init USB, which makes
    # cdc-acm renumber — an fd opened against the pre-chainload
    # /dev/ttyACMN points at a dead device and every read/write
    # returns EIO. See _open_vuart() below, called post-bootstrap.
    stop = threading.Event()
    verbose = os.environ.get("SPHRAGIS_HV_VERBOSE", "1") != "0"

    def _resolve_vuart_path():
        override = os.environ.get("SPHRAGIS_HV_VUART_DEVICE")
        if override:
            return override
        import glob
        by_id = glob.glob(
            "/dev/serial/by-id/usb-Asahi_Linux_m1n1_uartproxy*if02")
        if by_id:
            # Dereference the symlink. Opening through /dev/serial/by-id
            # triggers a different cdc-acm code path that returns EPROTO
            # on modem-control ioctls — safer to use the real node.
            return os.path.realpath(by_id[0])
        return "/dev/ttyACM2"

    def _open_vuart():
        last_err = None
        deadline = time.time() + 30
        while time.time() < deadline:
            path = _resolve_vuart_path()
            try:
                v = serial.Serial(
                    path,
                    baudrate=115200,
                    timeout=0.1,
                    rtscts=False, xonxoff=False, dsrdtr=False,
                )
                log(f"opened vuart at {path}")
                configure_vuart_raw(v)
                return v
            except (FileNotFoundError, serial.SerialException) as e:
                last_err = e
                time.sleep(1.0)
        log(f"timed out waiting for vuart: {last_err}")
        sys.exit(1)

    # M1N1DEVICE points at the uartproxy endpoint (-if00). Resolve
    # through /dev/serial/by-id so we survive USB re-enumerations
    # (device numbers shift; the by-id name is stable). We dereference
    # the symlink to the real /dev/ttyACMN path because m1n1's
    # UartInterface opens this directly via pyserial and some kernel
    # cdc-acm paths hang when opening through the symlink.
    if "M1N1DEVICE" not in os.environ:
        import glob
        proxy_symlinks = glob.glob(
            "/dev/serial/by-id/usb-Asahi_Linux_m1n1_uartproxy*if00")
        if proxy_symlinks:
            os.environ["M1N1DEVICE"] = os.path.realpath(proxy_symlinks[0])
        else:
            os.environ["M1N1DEVICE"] = "/dev/ttyACM1"
        log(f"M1N1DEVICE={os.environ['M1N1DEVICE']}")
    iface = UartInterface()
    # Clear HUPCL on the proxy fd too so DTR stays asserted across
    # our process lifetime. Same rationale as configure_vuart_raw.
    try:
        _clear_hupcl_and_set_raw(iface.dev.fileno())
        log("cleared HUPCL on iface (ACM1) fd")
    except Exception as e:
        log(f"iface HUPCL clear failed (non-fatal): {e!r}")
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)

    # SPHRAGIS_HV_BOOTSTRAP_CHAINLOAD=1 — do one chainload_inline at
    # startup so we're guaranteed to be talking to our patched m1n1
    # (the one built from this tree, with P_HV_MAP_VUART_DOCKCHANNEL
    # compiled in). Without this the Mac may still be running whatever
    # m1n1 kmutil-configure-boot installed, which on some systems
    # pre-dates the dockchannel-vuart opcode and makes hv.init() fail
    # with `ProxyCommandError: Reply error: Bad Command`. Harmless if
    # the running m1n1 already matches our build — just a 3 s reboot.
    if os.environ.get("SPHRAGIS_HV_BOOTSTRAP_CHAINLOAD", "0") == "1":
        boot_macho = os.environ.get(
            "SPHRAGIS_HV_CHAINLOAD_MACHO",
            str(pathlib.Path(__file__).resolve().parents[2] /
                "external/m1n1/build/m1n1.macho"))
        log(f"bootstrap: chainloading patched m1n1 from {boot_macho}")
        # chainload_inline needs a ProxyUtils to read chosen/memory-map.
        # Build a throwaway one against the current (possibly stale) m1n1;
        # the real u/hv get built below once we know we're on our binary.
        _tmp_u = ProxyUtils(p, heap_size=16 * 1024 * 1024)
        chainload_inline(iface, p, _tmp_u, boot_macho)
        try:
            iface.nop()
            log("bootstrap chainload ok — proxy talking to patched m1n1")
        except Exception as e:
            log(f"bootstrap chainload: post-reload nop failed: {e!r}")
            raise
        # Give Linux a moment to re-enumerate USB so the by-id symlink
        # repopulates before we resolve it below.
        time.sleep(1.5)

    # NOW open the vuart (post-chainload, so we resolve against the
    # live /dev/ttyACMN — may have shifted, e.g. ACM2 → ACM3).
    vuart = _open_vuart()
    _vuart_ref["vuart"] = vuart

    # Expose iface to vuart_reader before spawning it (so the reader's
    # halt-detection kick-writer can reach the proxy).
    _iface_for_kick["iface"] = iface

    # Deferred thread spawns — vuart is live now. Threads read vuart
    # from the ref so main can swap it across chainload boundaries.
    threading.Thread(target=vuart_reader, args=(_vuart_ref,),
                     daemon=True).start()
    if stims:
        threading.Thread(target=stimulus_sender,
                         args=(_vuart_ref, stims, verbose),
                         daemon=True).start()

    u, hv = _build_hv(iface, p)

    # Fast-iteration mode: dump ADT + hold proxy alive without
    # starting the HV. Useful for M4 coprocessor topology probing
    # where we don't need Sphragis running at all.
    if os.environ.get("SPHRAGIS_HV_ADT_ONLY", "0") == "1":
        _dump_keyboard_adt(u.adt)
        log("ADT_ONLY=1 — dumped ADT; holding proxy open. Ctrl+C / kill -9 to exit.")
        try:
            while True:
                time.sleep(5)
                try:
                    p.nop()
                except Exception as e:
                    log(f"adt-only: nop failed: {e!r}")
        except KeyboardInterrupt:
            pass
        return

    # Mac-keyboard-via-MTP probe. Sets up SMC, DART-MTP, MTP
    # dockchannel, and the MTP ASC coprocessor, then subscribes to
    # keyboard events. Each keystroke you make on the Mac keyboard
    # prints to stderr as the HID report + decoded ASCII. Also
    # writes the decoded byte to the vuart if SPHRAGIS_HV_MTP_BRIDGE_TO_VUART=1.
    # Does NOT start the HV — this is a pure keyboard probe.
    if os.environ.get("SPHRAGIS_HV_MTP_KBD_PROBE", "0") == "1":
        _mtp_kbd_probe(iface, p, u, vuart)
        return

    # SPHRAGIS_HV_LOOP=1 turns the whole halt → chainload → relaunch cycle
    # into an infinite Sphragis demo reel. One Python invocation keeps the
    # pyserial fds open across iterations (the wedge we spent a full
    # session chasing on 2026-04-20), so there's no physical power-cycle
    # between runs. Ctrl+C exits the loop cleanly; SPHRAGIS_HV_LOOP_MAX
    # caps iterations for smoke-tests / CI.
    loop = os.environ.get("SPHRAGIS_HV_LOOP", "0") == "1"
    max_iter = int(os.environ.get("SPHRAGIS_HV_LOOP_MAX", "0"))
    macho_path = os.environ.get(
        "SPHRAGIS_HV_CHAINLOAD_MACHO",
        str(pathlib.Path(__file__).resolve().parents[2] /
            "external/m1n1/build/m1n1.macho"))

    payload_path = os.environ.get("SPHRAGIS_HV_PAYLOAD", str(SPHRAGIS_BINARY))
    prestart_s = int(os.environ.get("SPHRAGIS_HV_PRESTART_SLEEP", "0"))
    diag_mode = os.environ.get("SPHRAGIS_HV_POST_EXIT_DIAG", "1") != "0"

    # stdin_forwarder started once; it runs across all iterations so a
    # live operator keeps typing to whichever session is current.
    if sys.stdin.isatty():
        threading.Thread(target=stdin_forwarder,
                         args=(_vuart_ref, stop), daemon=True).start()

    iteration = 0
    user_interrupted = False
    fatal_error = False
    while True:
        if iteration > 0:
            # Rebuild ProxyUtils + HV against the fresh m1n1 booted by
            # chainload_inline at end of previous iter. iface/p stay alive
            # (no pyserial reopens); u/hv hold stale heap/adt/bootargs
            # pointers into the PREVIOUS m1n1 image, so they must go.
            log(f"[iter {iteration}] building fresh ProxyUtils + HV")
            u, hv = _build_hv(iface, p)
            # Re-spawn stim thread so canned demo input fires for this
            # iteration too. Overlap with a previous iter's thread is
            # theoretically possible if iter 0 halts before stims finish
            # firing; in practice stim total time << boot+halt time.
            if stims:
                threading.Thread(target=stimulus_sender,
                                 args=(_vuart_ref, stims, verbose),
                                 daemon=True).start()

        hv.init()

        if iteration == 0 and os.environ.get(
                "SPHRAGIS_HV_DUMP_KBD_ADT", "0") == "1":
            _dump_keyboard_adt(hv.adt)

        # Allow overriding payload for diagnostic experiments (e.g. a
        # wfi-forever stub to isolate whether guest activity matters).
        log(f"[iter {iteration}] loading {payload_path}")
        hv.load_raw(pathlib.Path(payload_path).read_bytes(), 0)

        # Optional pre-start wall-clock delay for watchdog-source
        # experiments. First iteration only — loop iters start
        # immediately after chainload.
        if iteration == 0 and prestart_s > 0:
            log(f"SPHRAGIS_HV_PRESTART_SLEEP={prestart_s}s — waiting "
                f"before hv.start()")
            time.sleep(prestart_s)

        # Optional "init only" — call hv.init() but never hv.start().
        # First iteration only; doesn't interact with loop semantics.
        if iteration == 0 and os.environ.get(
                "SPHRAGIS_HV_INIT_ONLY", "0") == "1":
            log("SPHRAGIS_HV_INIT_ONLY=1 — not calling hv.start(); "
                "sleeping 200s")
            for i in range(200):
                if i % 10 == 0:
                    log(f"init-only t={i}s")
                time.sleep(1)
            log("init-only done")
            stop.set()
            vuart.close()
            return

        log(f"[iter {iteration}] calling hv.start() — Sphragis takes "
            f"over now. Ctrl+] to detach.")
        try:
            hv.start()
            # Reached if the last guest CPU exits cleanly (halt_sphragis
            # emits the halt marker → vuart_reader kicks → run_shell
            # intercepts → EXIT_GUEST → m1n1's hv_exit_guest unwinds
            # → hv_start returns). Mac SHOULD now be back in stock
            # proxy-serving state.
            log(f"[iter {iteration}] hv.start() returned cleanly "
                f"— probing post-HV proxy state.")
            if diag_mode:
                _post_exit_diag(p, iteration)
        except KeyboardInterrupt:
            user_interrupted = True
        except Exception as e:
            sys.stderr.write(
                f"\n[host] [iter {iteration}] hv.start() raised: {e}\n")
            sys.stderr.flush()
            fatal_error = True

        if user_interrupted or fatal_error or not loop:
            break
        iteration += 1
        if max_iter and iteration >= max_iter:
            log(f"hit SPHRAGIS_HV_LOOP_MAX={max_iter} — stopping loop")
            break

        log(f"[iter {iteration - 1} → {iteration}] chainloading fresh "
            f"m1n1 from {macho_path}")
        try:
            chainload_inline(iface, p, u, macho_path)
        except Exception as e:
            log(f"chainload failed: {e!r} — exiting loop")
            import traceback
            traceback.print_exc()
            break
        try:
            iface.nop()
        except Exception as e:
            log(f"post-chainload iface.nop failed: {e!r} — exiting loop")
            break

        # chainload_inline re-enumerates USB — the old vuart fd now
        # points at a dead /dev/ttyACMN. Close it (best-effort) and
        # open a fresh one. The ref update makes vuart_reader,
        # stim thread, and stdin_forwarder all pick up the new fd.
        try:
            vuart.close()
        except Exception:
            pass
        time.sleep(1.5)  # let /dev/serial/by-id repopulate
        vuart = _open_vuart()
        _vuart_ref["vuart"] = vuart

        # Re-arm halt detection so vuart_reader fires again for this iter.
        _halt_seen.clear()
        log(f"[iter {iteration}] fresh m1n1 ready, halt flag cleared, "
            f"relaunching Sphragis")

    stop.set()

    # Loop mode ends with the Mac in a clean m1n1 proxy state (last
    # hv.start() returned cleanly, post-exit probes passed). If we
    # let pyserial GC close the fds, DTR drops and wedges the USB CDC
    # — the next cold launch then needs a physical power-cycle. Skip
    # the close so the Mac stays warm for Ctrl+C→relaunch demos.
    # Explicit SPHRAGIS_HV_NO_CLOSE=1 forces the same behaviour outside
    # loop mode.
    if loop or os.environ.get("SPHRAGIS_HV_NO_CLOSE", "0") == "1":
        log("detaching via os._exit(0) — skipping pyserial close "
            f"(loop={loop})")
        sys.stdout.flush()
        sys.stderr.flush()
        os._exit(0)

    # Diagnostic: keep the proxy connection held open indefinitely.
    # Ignored in loop mode (the loop is itself a superset).
    if not loop and os.environ.get("SPHRAGIS_HV_HOLD_OPEN", "0") == "1":
        log("HOLD_OPEN=1 — keeping proxy fds alive. Poke me with kill -9 to exit.")
        try:
            while True:
                time.sleep(5)
                try:
                    p.nop()
                    log("hold-open: p.nop() ok")
                except Exception as e:
                    log(f"hold-open: nop failed: {e!r}")
        except KeyboardInterrupt:
            pass

    # One-shot chainload + hold (pre-dates SPHRAGIS_HV_LOOP). Skipped in
    # loop mode since the loop already did any chainloads it wanted.
    if not loop and os.environ.get("SPHRAGIS_HV_RECHAINLOAD", "0") == "1":
        log(f"re-chainloading {macho_path} within this session")
        try:
            chainload_inline(iface, p, u, macho_path)
            log("re-chainload OK — proxy is talking to a fresh m1n1. "
                "Holding session open; attach tools via proxy API.")
            while True:
                time.sleep(5)
                try:
                    p.nop()
                    log("post-chainload: p.nop() ok")
                except Exception as e:
                    log(f"post-chainload: nop failed: {e!r}")
        except Exception as e:
            log(f"re-chainload failed: {e!r}")
            import traceback
            traceback.print_exc()

    log("detaching — draining vuart for 2s")
    time.sleep(2)
    vuart.close()


if __name__ == "__main__":
    main()
