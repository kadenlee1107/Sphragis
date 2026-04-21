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
    looks exactly like Bat_OS is broken.

    Also clears HUPCL so the kernel DOESN'T drop DTR when the last
    fd closes. On USB-CDC the Mac's m1n1 interprets DTR-toggle as
    "host disconnected" and wedges its USB stack — any subsequent
    pyserial.Serial() open against /dev/ttyACM* then blocks forever
    even though the device node still exists. Clearing HUPCL keeps
    modem-control lines steady across our process lifetime."""
    vuart.dtr = True
    vuart.rts = False
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


def _dump_keyboard_adt(adt):
    """Walk the ADT looking for keyboard/HID-transport nodes and
    their containing SPI controllers. Writes an annotated listing
    to stderr + /tmp/adt_kbd.log so we can pick the right MMIO
    base for Bat_OS's SPI keyboard driver.

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
    # MTP (MultiTouch Protocol). Find the AOP coprocessor, the MTP
    # node, their DART, and anything with "mtp" or "aop" in the name
    # or compat.
    pline("")
    pline("[adt-kbd] AOP / MTP coprocessor nodes (M4 keyboard path):")
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
            if ("mtp" in nm_l or "aop" in nm_l or "mtp" in c_join or
                "aop" in c_join or "rtkit" in c_join or
                any("hid" in c.lower() for c in compat)):
                try:
                    parent = getattr(node, "_parent", None)
                    ppath = parent.name if parent else "?"
                except Exception:
                    ppath = "?"
                pline(f"  {ppath}/{nm}  compatible={compat}")
                try:
                    reg = node.get_reg(0)
                    pline(f"    reg[0] = (0x{reg[0]:x}, 0x{reg[1]:x})")
                except Exception:
                    pass
                try:
                    irqs = getattr(node, "interrupts", None)
                    if irqs:
                        pline(f"    interrupts = {list(irqs)}")
                except Exception:
                    pass
                # Print some property keys
                try:
                    props = [k for k in dir(node) if not k.startswith("_")
                             and not callable(getattr(node, k, None))]
                    interesting = [k for k in props if any(
                        s in k.lower() for s in
                        ["endpoint", "rtk", "mbox", "asc", "iommu", "dart",
                         "function", "ep-", "ep_"])]
                    if interesting:
                        pline(f"    props-of-interest: {interesting[:12]}")
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
    from m1n1.proxy import IODEV
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
    # Clear HUPCL on the proxy fd too so DTR stays asserted across
    # our process lifetime. Same rationale as configure_vuart_raw.
    try:
        _clear_hupcl_and_set_raw(iface.dev.fileno())
        log("cleared HUPCL on iface (ACM1) fd")
    except Exception as e:
        log(f"iface HUPCL clear failed (non-fatal): {e!r}")
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

    if os.environ.get("BATOS_HV_DUMP_KBD_ADT", "0") == "1":
        _dump_keyboard_adt(hv.adt)

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
        # Reached if the last guest CPU exits cleanly (halt_bat_os
        # emits the halt marker → vuart_reader kicks → run_shell
        # intercepts → EXIT_GUEST → m1n1's hv_exit_guest unwinds →
        # hv_start returns). Mac SHOULD now be back in stock
        # proxy-serving state.
        log("hv.start() returned cleanly — probing post-HV proxy state.")

        # Post-HV diagnostic probes — narrows the "chainload fails
        # after clean exit" wedge. Each probe reports alive/dead so
        # we can tell which layer's stuck.
        diag_mode = os.environ.get("BATOS_HV_POST_EXIT_DIAG", "1") != "0"
        if diag_mode:
            # 1. Most basic: can the proxy still process a NOP?
            try:
                p.nop()
                log("post-exit probe: p.nop() OK")
            except Exception as e:
                log(f"post-exit probe: p.nop() FAIL: {e!r}")

            try:
                base = p.get_base()
                log(f"post-exit probe: p.get_base()=0x{base:x}")
            except Exception as e:
                log(f"post-exit probe: p.get_base() FAIL: {e!r}")

            # 2. Try resetting vuart iodev back to console mode.
            # USB_VUART was reconfigured by hv_map_vuart_dockchannel;
            # restoring USAGE_CONSOLE may let the outer proxy scan see it.
            try:
                from m1n1.proxy import IODEV, USAGE
                # Re-enable console + uartproxy on the USB_VUART endpoint
                p.iodev_set_usage(IODEV.USB_VUART,
                                  USAGE.CONSOLE | USAGE.UARTPROXY)
                log("post-exit probe: iodev_set_usage(USB_VUART, CONSOLE|UARTPROXY) OK")
            except Exception as e:
                log(f"post-exit probe: iodev_set_usage FAIL: {e!r}")

            # 3. Optionally shut down FB (if BATOS_KEEP_FB left it live)
            if os.environ.get("BATOS_HV_POST_EXIT_FB_SHUTDOWN", "0") == "1":
                try:
                    p.fb_shutdown(True)
                    log("post-exit probe: p.fb_shutdown() OK")
                except Exception as e:
                    log(f"post-exit probe: p.fb_shutdown() FAIL: {e!r}")
    except KeyboardInterrupt:
        pass
    except Exception as e:
        sys.stderr.write(f"\n[host] hv.start() raised: {e}\n")
        sys.stderr.flush()

    stop.set()

    # Diagnostic: if set, skip all pyserial cleanup (vuart.close() and
    # Python's normal GC-driven iface.dev close). On exit the kernel
    # drops the fds, but without DTR-toggle / flush sequences that
    # pyserial invokes on close — which seem to put m1n1's USB CDC
    # state into a mode where fresh pyserial opens time out.
    if os.environ.get("BATOS_HV_NO_CLOSE", "0") == "1":
        log("detaching via os._exit(0) — skipping pyserial close")
        sys.stdout.flush()
        sys.stderr.flush()
        os._exit(0)

    # Diagnostic: keep the proxy connection held open indefinitely.
    # Used to test whether it's the pyserial/kernel close that wedges
    # the Mac vs. something we did earlier. Kill with SIGINT/kill -9.
    if os.environ.get("BATOS_HV_HOLD_OPEN", "0") == "1":
        log("HOLD_OPEN=1 — keeping proxy fds alive. Poke me with kill -9 to exit.")
        # Ping m1n1 periodically so the Mac can verify we're alive.
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

    # Experimental: re-chainload a fresh m1n1 using the existing
    # proxy session. Proven earlier: a SECOND pyserial process can't
    # open /dev/ttyACM1 while ours holds it — so fresh chainload.py
    # from a shell hangs. Doing chainload INSIDE this session reuses
    # iface/p/u, no open conflict, no DTR drop.
    if os.environ.get("BATOS_HV_RECHAINLOAD", "0") == "1":
        macho_path = os.environ.get(
            "BATOS_HV_CHAINLOAD_MACHO",
            str(pathlib.Path(__file__).resolve().parents[2] /
                "external/m1n1/build/m1n1.macho"))
        log(f"re-chainloading {macho_path} within this session")
        try:
            chainload_inline(iface, p, u, macho_path)
            log("re-chainload OK — proxy is talking to a fresh m1n1. "
                "Holding session open; attach tools via proxy API.")
            # Keep the session alive so the tty doesn't drop DTR.
            while True:
                time.sleep(5)
                try:
                    p.nop()
                    log("post-chainload: p.nop() ok")
                except Exception as e:
                    log(f"post-chainload: nop failed: {e!r}")
        except Exception as e:
            log(f"re-chainload failed: {e!r}")
            import traceback; traceback.print_exc()

    log("detaching — draining vuart for 2s")
    time.sleep(2)
    vuart.close()


if __name__ == "__main__":
    main()
