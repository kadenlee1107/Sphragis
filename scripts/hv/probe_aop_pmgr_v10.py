#!/usr/bin/env python3
"""v10: pure observation — RUN=1 without disturbing any AOP state.

Hypothesis: if iBoot left AOP's PAC keys set in the APIA/APIB
sysregs and we do NOT overwrite __DATA or bootargs (which might
corrupt FW runtime state), a simple RUN=1 might let FW resume
from its pre-halt state and Hello.

Rules:
  - Chainload patched m1n1 (we need WDT fix)
  - DAPF init (required per m1n1 to program fabric)
  - SKIP __DATA write (iBoot has it)
  - SKIP __OS_LOG write (iBoot has it)
  - SKIP update_bootargs (use iBoot's defaults)
  - SKIP OUTBOX_CTRL reset
  - SKIP INBOX write
  - SKIP doorbell
  - Just set CC.RUN=1 and watch for 20 s
"""
import os, pathlib, struct, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port
from m1n1 import asm

M1N1_MACHO  = ROOT / "external/m1n1/build/m1n1.macho"
AOP = 0x390600000

CPU_CONTROL  = 0x0044
CPU_STATUS   = 0x0048
INBOX_CTRL   = 0x8110
OUTBOX_CTRL  = 0x8114
OUTBOX0      = 0x8830
OUTBOX1      = 0x8838
EMPTY_BIT    = 1 << 17


def log(m): print(f"[v10] {m}", flush=True)


def snap(p, tag):
    cc = p.read32(AOP + CPU_CONTROL)
    cs = p.read32(AOP + CPU_STATUS)
    ic = p.read32(AOP + INBOX_CTRL)
    oc = p.read32(AOP + OUTBOX_CTRL)
    log(f"  [{tag}] CC={cc:#x} CS={cs:#x} IB={ic:#x} OB={oc:#x}")


def chainload(iface, p, u):
    from m1n1.macho import MachO
    from m1n1.tgtypes import BootArgs_r1, BootArgs_r2, BootArgs_r3
    from m1n1.utils import align
    new_base = u.base
    data = M1N1_MACHO.read_bytes()
    macho = MachO(data)
    image = macho.prepare_image() + b"\x00\x00\x00\x00"
    entry = macho.entry - macho.vmin + new_base
    sepfw_start, sepfw_length = u.adt["chosen"]["memory-map"].SEPFW
    preoslog_start, preoslog_size = 0, 0
    if hasattr(u.adt["chosen"]["memory-map"], "preoslog"):
        preoslog_start, preoslog_size = u.adt["chosen"]["memory-map"].preoslog
    image_size = align(len(image))
    sepfw_off = image_size; image_size += align(sepfw_length)
    preoslog_off = image_size; image_size += align(preoslog_size)
    bootargs_off = image_size; image_size += 0x4000
    image_addr = u.malloc(image_size)
    u.compressed_writemem(image_addr, image, True)
    p.dc_cvau(image_addr, len(image))
    p.memcpy8(image_addr + sepfw_off, sepfw_start, sepfw_length)
    u.adt["chosen"]["memory-map"].SEPFW = (new_base + sepfw_off, sepfw_length)
    u.adt["chosen"]["memory-map"].BootArgs = (new_base + bootargs_off, 0x4000)
    if preoslog_size:
        p.memcpy8(image_addr + preoslog_off, preoslog_start, preoslog_size)
        u.adt["chosen"]["memory-map"].preoslog = (new_base + preoslog_off, preoslog_size)
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
1:      ldp x4, x5, [x1], #16
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
    p.reload(stub.addr, new_base + bootargs_off, image_addr, new_base, image_size)
    iface.nop()


def main():
    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    log("chainloading patched m1n1 (WDT fix only)...")
    chainload(iface, p, u)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)
    log("  up")

    # DAPF init (m1n1 needs this for fabric)
    try:
        p.dapf_init("/arm-io/dart-aop")
        log("dapf OK")
    except Exception as e:
        log(f"dapf: {e!r}")

    snap(p, "raw pre-RUN (iBoot state)")

    # Dump a few bytes of iBoot-populated bootargs area to confirm
    # they're non-default (iBoot filled them)
    try:
        from m1n1.fw.aop.base import AOPBase
        aopb = AOPBase(u)
        args = aopb.read_bootargs()
        log(f"iBoot bootargs keys: {list(args.keys())[:10]}")
        for k in ['p0CE', 'laCn', 'gila', 'tPOA']:
            if k in args:
                log(f"    {k} = {args[k].hex()}")
    except Exception as e:
        log(f"read_bootargs: {type(e).__name__}: {e}")

    # Just flip RUN=1. No INBOX write. No doorbell.
    log("\nCC.RUN=1 (pure, no other writes)")
    p.write32(AOP + CPU_CONTROL, 0x10)

    log("\nobserve 20 s (watch for ANY state change or OUTBOX msg)...")
    msgs = []
    deadline = time.time() + 20.0
    last_state = None
    while time.time() < deadline:
        try:
            cc = p.read32(AOP + CPU_CONTROL)
            cs = p.read32(AOP + CPU_STATUS)
            ic = p.read32(AOP + INBOX_CTRL)
            oc = p.read32(AOP + OUTBOX_CTRL)
        except Exception as e:
            log(f"  proxy err: {e!r}")
            break
        state = (cc, cs, ic, oc)
        if state != last_state:
            t = 20 - (deadline - time.time())
            log(f"  t={t:5.2f}s CC={cc:#x} CS={cs:#x} IB={ic:#x} OB={oc:#x}")
            last_state = state
        if not (oc & EMPTY_BIT):
            m0 = p.read32(AOP + OUTBOX0) | (p.read32(AOP + OUTBOX0 + 4) << 32)
            m1 = p.read32(AOP + OUTBOX1) | (p.read32(AOP + OUTBOX1 + 4) << 32)
            log(f"  *** AOP MSG m0={m0:#x} m1={m1:#x} TYPE={(m0>>52)&0xff:#x} ***")
            msgs.append((m0, m1))
            time.sleep(0.005)
        time.sleep(0.05)

    snap(p, "final")
    log(f"\nmsgs: {len(msgs)}")
    if msgs:
        log("*** AOP SELF-HELLOED from iBoot's preserved state ***")
    else:
        log("AOP silent — iBoot did not leave it resumable")
    os._exit(0 if msgs else 2)


if __name__ == "__main__":
    main()
