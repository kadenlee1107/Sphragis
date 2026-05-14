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
  SPHRAGIS_LINKALIAS=0                       — forced off (we're XNU, not Sphragis)
  SPHRAGIS_KEEP_FB=0                         — forced off (XNU owns the FB)

Exits to an HV shell after guest stops (panic, ^C, or completion). Look
at MTP_TRACE_LOG for the captured MMIO sequence.
"""
import os
import pathlib
import sys
import traceback

# --- Force HV flags appropriate for XNU guest BEFORE importing m1n1.hv
# hv/__init__.py reads these at start() time; setting them in argv or env
# here is the cleanest way to override what the Sphragis chainload defaults
# would otherwise set. SPHRAGIS_KEEP_FB defaults to 0 (XNU owns the FB) but
# we let the user override it — some XNU early-boot paths may panic if
# the FB goes away mid-init.
os.environ["SPHRAGIS_LINKALIAS"] = "0"
os.environ.setdefault("SPHRAGIS_KEEP_FB", "0")

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

    # Dump EL1 sysregs automatically on guest fault. The HV's existing
    # context dump (from handle_exception) shows us the FINAL state
    # where XNU was running when it bounced to EL2 — but if XNU took an
    # EL1-internal sync exception and then faulted again trying to
    # vector through VBAR_EL1, that dump tells us nothing about the
    # ORIGINAL fault. The EL12-aliased registers are still set to the
    # values XNU wrote, so we can read them after the HV has caught
    # control and know exactly what XNU was doing at the first failure.
    #
    # Encodings via (op0, op1, CRn, CRm, op2) tuples — u.mrs parses
    # these directly. EL12 aliases = op1=5 (E2H-relative EL1 access).
    EL1_REGS = [
        ("SCTLR_EL12",  (3, 5, 1, 0, 0)),   # is MMU on? caches? WXN?
        ("TTBR0_EL12",  (3, 5, 2, 0, 0)),   # user page table
        ("TTBR1_EL12",  (3, 5, 2, 0, 1)),   # kernel page table
        ("TCR_EL12",    (3, 5, 2, 0, 2)),   # translation control
        ("MAIR_EL12",   (3, 5, 10, 2, 0)),
        ("CPACR_EL12",  (3, 5, 1, 0, 2)),   # SIMD/FP enable
        ("SPSR_EL12",   (3, 5, 4, 0, 0)),
        ("ELR_EL12",    (3, 5, 4, 0, 1)),   # PC that took exception
        ("ESR_EL12",    (3, 5, 5, 2, 0)),   # syndrome
        ("FAR_EL12",    (3, 5, 6, 0, 0)),   # fault address
        ("VBAR_EL12",   (3, 5, 12, 0, 0)),  # exception base
        ("SP_EL1",      (3, 4, 4, 1, 0)),   # guest kernel stack
        ("TPIDR_EL1",   (3, 5, 13, 0, 4)),  # per-CPU pointer
        ("CNTKCTL_EL12", (3, 5, 14, 1, 0)),
    ]

    def _dump_el1_regs(tag: str = "post-fault"):
        print(f"\n=== EL1 sysreg dump ({tag}) ===")
        for name, enc in EL1_REGS:
            try:
                v = hv.u.mrs(enc, silent=True)
                print(f"  {name:14s} = 0x{v:016x}")
            except Exception as e:
                print(f"  {name:14s} = <err: {type(e).__name__}: {e}>")
        print("====================================\n", flush=True)

    # Hook hv.handle_exception so the dump happens at the moment the
    # exception is received, BEFORE print_context tries to disasm the
    # faulting ELR (which can hang when guest VAs aren't translatable).
    _orig_handle_exception = hv.handle_exception

    def _handle_exception_with_dump(reason, code, info):
        try:
            # dump immediately so we get the data even if downstream
            # print_context wedges on an unmappable ELR
            _dump_el1_regs(f"at exception reason={reason} code={code}")
        except Exception:
            traceback.print_exc()
        return _orig_handle_exception(reason, code, info)

    hv.handle_exception = _handle_exception_with_dump
    # Re-register with iface since set_handler captured the original
    # bound method at hv.init() time.
    from m1n1.proxy import START, EXC
    iface.set_handler(START.EXCEPTION_LOWER, EXC.SYNC,   _handle_exception_with_dump)
    iface.set_handler(START.EXCEPTION_LOWER, EXC.IRQ,    _handle_exception_with_dump)
    iface.set_handler(START.EXCEPTION_LOWER, EXC.FIQ,    _handle_exception_with_dump)
    iface.set_handler(START.EXCEPTION_LOWER, EXC.SERROR, _handle_exception_with_dump)

    # Patch the whole __TEXT_BOOT_EXEC prelude (offsets 0x00..0x1f, 8
    # insns). Two reasons iBoot-simulation is necessary:
    #   1. m1n1's hv_enter_guest zeroes SP_EL1 → first `stp [sp,#-0x10]!`
    #      in the bootstrap handler at +0x4000 wraps to 0xFFFF...FFF0
    #      (data abort, ESR=0x96000040 observed on v6).
    #   2. SCTLR_EL1.EnIA/EnIB start 0 at ERET → `pacibsp` is a NOP and
    #      saved LRs are unsigned. When XNU's init later sets En*B=1 (or
    #      uses blraa/braa to branch to PAC-signed function pointers
    #      already encoded in data tables), the mismatched auth yields
    #      a garbage PC — observed on v7 as ELR=FAR=0x8010d7e1019ffe5c
    #      (PAC-decorated), ESR=0x86000000 IABORT_CURRENT_EL address-
    #      size fault.
    # The secondary-CPU handshake this prelude does (store w2 at a
    # progress-flag address) only matters if we start other CPUs, which
    # we don't. Safe to overwrite the whole range.
    #
    # New 8-insn prelude (verified via capstone):
    #   mrs   x9, sctlr_el1                   ; 09 10 38 d5
    #   mov   x10, #0xc0000000                ; 0a 00 b8 d2  (EnIA|EnIB)
    #   orr   x9, x9, x10                     ; 29 01 0a aa
    #   msr   sctlr_el1, x9                   ; 09 10 18 d5
    #   isb                                   ; df 3f 03 d5
    #   mov   sp, x3                          ; 7f 00 00 91  (SP from ERET x3)
    #   mov   x0, x1                          ; e0 03 01 aa  (preserve bootargs)
    #   b     #0x4000                         ; f9 0f 00 14  (real bootstrap)
    PATCH_BYTES = bytes.fromhex(
        "091038d5" "0a00b8d2" "29010aaa" "091018d5"
        "df3f03d5" "7f000091" "e00301aa" "f90f0014"
    )
    PATCH_OFFSET = 0x00          # within __TEXT_BOOT_EXEC
    entry_pa = hv.entry          # load_macho set hv.entry to guest PA
    patch_pa = entry_pa + PATCH_OFFSET
    print(f"Patching entry prelude at {patch_pa:#x} ({len(PATCH_BYTES)} bytes)")
    iface.writemem(patch_pa, PATCH_BYTES)
    p.dc_cvau(patch_pa, len(PATCH_BYTES))
    p.ic_ivau(patch_pa, len(PATCH_BYTES))

    # Pick a safe SP for bootstrap. Above top_of_kernel_data so the
    # first `stp [sp, #-0x10]!` writes inside mapped RAM but outside
    # anything XNU expects to already be populated.
    bootstrap_sp = hv.tba.top_of_kernel_data + 0x20000   # 128 KiB pad
    bootstrap_sp &= ~0xf                                   # 16-B align
    # Zero-fill the top of the stack so PAC auth of LR on return doesn't
    # read uninitialized data if XNU ever uses SP-relative loads.
    iface.writemem(bootstrap_sp - 0x1000, bytes(0x1000))

    # Monkey-patch hv.start's proxy call so the guest enters with the
    # boot-CPU register convention observed in macOS 26.3 J604
    # kernelcache disasm (see docs/SESSION_JOURNAL.md):
    #     x0 = 4              bootstrap-CPU magic
    #     x1 = bootargs_ptr
    #     x2 = flag byte
    #     x3 = bootstrap_sp   ← new: valid EL1 SP for first stack push
    if os.environ.get("XNU_BOOT_CPU_ID") != "":  # default on; "" to disable
        orig_hv_start = hv.p.hv_start
        bootargs_ptr = hv.guest_base + hv.bootargs_off

        def patched_hv_start(entry, _old_bootargs_ptr_unused):
            cpu_id = int(os.environ.get("XNU_BOOT_CPU_ID", "4"))
            flag   = int(os.environ.get("XNU_BOOT_FLAG",   "0"))
            print(f"hv_start override: x0={cpu_id} x1={bootargs_ptr:#x} "
                  f"x2={flag} x3={bootstrap_sp:#x} entry={entry:#x}")
            return orig_hv_start(entry, cpu_id, bootargs_ptr, flag,
                                 bootstrap_sp)

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
        _dump_el1_regs("post-KeyboardInterrupt")
    except Exception:
        traceback.print_exc()
        _dump_el1_regs("post-python-exception")

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
