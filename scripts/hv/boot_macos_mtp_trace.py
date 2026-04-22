#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""
Boot the J604 kernelcache as an HV guest on M4 and trace MTP/AOP MMIO.

Goal: record every MMIO read/write to the MTP + dart-mtp + dockchannel-mtp
register windows during macOS's native boot, so the resulting log can be
diffed against our raw-proxy attempts (scripts/hv/boot_mtp_dartmap.py).
The delta is the missing IOKit service-layer setup we couldn't replicate.

This is a thin wrapper around external/m1n1/proxyclient/tools/run_guest.py
that hardcodes:
  - payload   = macos_dump/kernelcache.mac16j.bin (filetype=12 fileset Mach-O)
  - trace set = hv/trace_mtp.py (ASC + DART + DockChannel, already in m1n1)

Usage:
  1. Fresh power cycle of the M4. This lands on the kmutil-installed
     m1n1 (Preboot). Whether that build already has the WDT fix depends
     on the last `./scripts/install-m1n1.sh`; if not, set WDT_KICK=1
     below to zero 0x3882BC224 before HV init.
  2. `sg dialout -c 'PYTHONUNBUFFERED=1 python3 scripts/hv/boot_macos_mtp_trace.py'`
     Add `--dry-run` to set everything up but drop to shell before ERET.

Env knobs:
  MTP_TRACE_LOG=/tmp/mtp_hv_trace.log    (default) — HV log file path
  TRACE_AOP=1                             — also load hv/trace_aop.py
  WDT_KICK=1                              — p.write32(0x3882BC224, 0) before init
                                            (mandatory on stock m1n1; no-op on
                                            patched m1n1 that already did it)
  KERNELCACHE=<path>                      — override default J604 kernelcache
  XNU_BOOTARGS="-v debug=0x8 serial=3"    — override iBoot-inherited bootargs
  HV_SMP=0                                — strip secondary CPUs from ADT (debug)
  BATOS_LINKALIAS=0                       — forced off (we're XNU, not Bat_OS)
  BATOS_KEEP_FB=0                         — forced off (XNU owns the FB)

Exits to an HV shell after guest stops (panic, ^C, or completion). Look
at MTP_TRACE_LOG for the captured MMIO sequence.
"""
import os
import pathlib
import sys
import traceback

# --- Force HV flags appropriate for XNU guest BEFORE importing m1n1.hv
# hv/__init__.py reads these at start() time; setting them in argv or env
# here is the cleanest way to override what the Bat_OS chainload defaults
# would otherwise set. BATOS_KEEP_FB defaults to 0 (XNU owns the FB) but
# we let the user override it — some XNU early-boot paths may panic if
# the FB goes away mid-init.
os.environ["BATOS_LINKALIAS"] = "0"
os.environ.setdefault("BATOS_KEEP_FB", "0")

# --- Path bootstrap identical to tools/run_guest.py so `from m1n1.*`
# resolves against the vendored proxyclient.
REPO = pathlib.Path(__file__).resolve().parents[2]
PROXYCLIENT = REPO / "external" / "m1n1" / "proxyclient"
sys.path.insert(0, str(PROXYCLIENT))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port
from m1n1.shell import run_shell
from m1n1.hv import HV

KERNELCACHE = pathlib.Path(
    os.environ.get("KERNELCACHE",
                   str(REPO / "macos_dump" / "kernelcache.mac16j.bin")))
TRACE_MTP_SCRIPT = PROXYCLIENT / "hv" / "trace_mtp.py"
TRACE_AOP_SCRIPT = PROXYCLIENT / "hv" / "trace_aop.py"

DRY_RUN = "--dry-run" in sys.argv
LOG_PATH = pathlib.Path(os.environ.get("MTP_TRACE_LOG", "/tmp/mtp_hv_trace.log"))
TRACE_AOP = os.environ.get("TRACE_AOP", "0") == "1"
WDT_KICK = os.environ.get("WDT_KICK", "0") == "1"
XNU_BOOTARGS = os.environ.get("XNU_BOOTARGS")  # None = inherit from iBoot

# M4 AP watchdog deadline-arm bit. Writing 0 here extends our budget
# past 118 s. Panic scratch regs share the same page (0x3882B8008,
# 0x3882B802C, 0x3882B8020) — DO NOT touch them or SMC dies.
M4_AP_WDT_DEADLINE = 0x3882BC224


def main() -> int:
    if not KERNELCACHE.exists():
        print(f"!! missing kernelcache: {KERNELCACHE}", file=sys.stderr)
        return 1

    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    if WDT_KICK:
        print(f"Zeroing M4 AP-WDT deadline-arm at {M4_AP_WDT_DEADLINE:#x}")
        p.write32(M4_AP_WDT_DEADLINE, 0)

    hv = HV(iface, p, u)
    if os.environ.get("HV_SMP") == "0":
        hv.smp = False
        print("HV: single-CPU mode (stripping secondaries from ADT)")
    hv.init()

    # Open the trace log up-front so every hv.log(...) / trace event lands
    # there. Flush aggressively — a bad HV exit shouldn't lose the tail.
    LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
    hv_log = LOG_PATH.open("w", buffering=1)
    hv.set_logfile(hv_log)
    print(f"HV log: {LOG_PATH}")

    # Load the kernelcache. FILESET Mach-O (filetype=12) is understood by
    # m1n1.macho.MachO.load_fileset(); the proxyclient already handles
    # Apple's post-iBoot bootkc format.
    print(f"Loading {KERNELCACHE} ({KERNELCACHE.stat().st_size} bytes)...")
    with KERNELCACHE.open("rb") as f:
        hv.load_macho(f)

    if XNU_BOOTARGS is not None:
        hv.set_bootargs(XNU_BOOTARGS)
    # else: self.tba.cmdline carries whatever iBoot passed to m1n1 — for a
    # Mac that boots macOS normally that's already the right thing.

    # Monkey-patch hv.start's proxy call so the guest enters with the
    # boot-CPU register convention observed in macOS 26.3 J604
    # kernelcache disasm (see docs/SESSION_JOURNAL.md):
    #     x0 = 4              (bootstrap-CPU magic at
    #                          __TEXT_BOOT_EXEC entry, offset 0x8)
    #     x1 = bootargs_ptr
    #     x2 = flag byte      (stored at adrp(...) + 0xf48 by
    #                          the entry-code strb prelude)
    # Without x0 == 4 XNU lands in its MPIDR-based CPU-lookup path and
    # spins forever on an unpopulated cpu_table (cbz x21, self at
    # 0xfffffe000c1bc0d4). m1n1's stock run_guest.py passes only
    # x0=bootargs_ptr, which is the old (pre-Darwin 26?) convention.
    if os.environ.get("XNU_BOOT_CPU_ID") != "":  # default on; "" to disable
        orig_hv_start = hv.p.hv_start
        bootargs_ptr = hv.guest_base + hv.bootargs_off

        def patched_hv_start(entry, _old_bootargs_ptr_unused):
            cpu_id = int(os.environ.get("XNU_BOOT_CPU_ID", "4"))
            flag   = int(os.environ.get("XNU_BOOT_FLAG",   "0"))
            print(f"hv_start override: x0={cpu_id} x1={bootargs_ptr:#x} "
                  f"x2={flag} entry={entry:#x}")
            return orig_hv_start(entry, cpu_id, bootargs_ptr, flag)

        hv.p.hv_start = patched_hv_start

    # Install tracers. `hv.run_script` evaluates the script in an env where
    # `hv`, `p`, `u`, `iface` are already injected (see hv.shell_locals).
    print(f"Installing MTP tracer: {TRACE_MTP_SCRIPT}")
    hv.run_script(str(TRACE_MTP_SCRIPT))

    if TRACE_AOP:
        print(f"Installing AOP tracer: {TRACE_AOP_SCRIPT}")
        hv.run_script(str(TRACE_AOP_SCRIPT))

    if DRY_RUN:
        print("--dry-run requested; dropping to HV shell (no ERET).")
        run_shell(hv.shell_locals,
                  "Dry-run HV shell. Inspect, then ^D to exit without starting guest.")
        return 0

    print("Starting guest (macOS kernelcache under HV) ...")
    try:
        hv.start()  # does not return until guest halts / ^C
    except KeyboardInterrupt:
        print("KeyboardInterrupt; guest stopped.")
    except Exception:
        traceback.print_exc()

    # After hv.start() returns we're back at EL2 with the guest halted.
    # Drop to a shell so we can inspect state and re-issue MMIO reads
    # that help diff against boot_mtp_dartmap.py.
    run_shell(hv.shell_locals, "Guest exited. HV shell (MMIO log at "
                                f"{LOG_PATH}).")

    p.smp_stop_secondaries(True)
    p.sleep(True)
    hv_log.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
