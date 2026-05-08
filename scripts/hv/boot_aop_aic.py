#!/usr/bin/env python3
"""Variant of boot_aop.py that masks AOP's AIC IRQs before aop.start().

Theory: on M4 ascwrap-v6, AOP FW ends up with CS=0x48 (IRQ_NOT_PEND=0 ->
some IRQ pending) because the AIC is routing AOP-generated IRQs somewhere
we don't ack. Mask them at AIC so the ASC CPU doesn't see pending state.

AIC3 layout on M4 (t8132):
  base = 0x381000000
  IRQ_CFG       = base + 0x10000     # 32b per IRQ, 4096 IRQs
  SW_SET        = base + 0x14000
  SW_CLR        = base + 0x14200
  MASK_SET      = base + 0x14400     # write 1 to mask (disable delivery)
  MASK_CLR      = base + 0x14600     # write 1 to unmask
  HW_STATE      = base + 0x14800
  stride per die = 0x4a00 (but nr_die=1 on M4)
  aic-iack-offset = 0x40000 (read to dequeue pending event)

AOP interrupts = [434, 433, 436, 435]
MTP interrupts = [1114, 1113, 1116, 1115]
dart-aop irq   = [457]

Variation modes (env BATOS_AOP_VAR):
  0 = baseline (same as boot_aop.py, no AIC fiddle)
  1 = AIC_MASK_SET all AOP IRQs before aop.start()
  2 = variation 1 + drain AIC EVENT reg before aop.start()
  3 = variation 2 + mask dart-aop IRQ too
"""
import os
import pathlib
import struct
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

AOP_BLOB = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
M1N1_MACHO = ROOT / "external/m1n1/build/m1n1.macho"

# AIC3 layout on t8132
AIC_BASE        = 0x381000000
AIC_IRQ_CFG     = 0x10000
AIC_SW_SET      = 0x14000
AIC_SW_CLR      = 0x14200
AIC_MASK_SET    = 0x14400
AIC_MASK_CLR    = 0x14600
AIC_HW_STATE    = 0x14800
AIC_EVENT       = 0x40000   # iack-offset


def log(m): print(f"[aop-aic] {m}", flush=True)


def chainload(iface, p, u):
    from m1n1.macho import MachO
    from m1n1.tgtypes import BootArgs_r1, BootArgs_r2, BootArgs_r3
    from m1n1 import asm
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
    sepfw_off = image_size
    image_size += align(sepfw_length)
    preoslog_off = image_size
    image_size += align(preoslog_size)
    bootargs_off = image_size
    image_size += 0x4000
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


def macho_segs_fn(mb):
    ncmds = struct.unpack("<I", mb[16:20])[0]
    cur, segs = 32, []
    for _ in range(ncmds):
        cmd, sz = struct.unpack("<II", mb[cur:cur+8])
        if cmd == 0x19:
            name = mb[cur+8:cur+24].rstrip(b"\x00").decode()
            vm, vmsz, fo, fs = struct.unpack("<QQQQ", mb[cur+24:cur+56])
            segs.append((name, vm, vmsz, fo, fs))
        cur += sz
    return segs


def parse_adt_segments(raw):
    out = []
    for i in range(len(raw) // 32):
        s = raw[i*32:(i+1)*32]
        phys, iova, remap, size = struct.unpack("<QQQI4x", s)
        out.append({"phys": phys, "iova": iova, "size": size})
    return out


def aic_mask_irq(p, irq):
    """Write 1 to AIC_MASK_SET bit (irq%32) at word (irq/32)*4."""
    reg_off = (irq >> 5) * 4
    bit = 1 << (irq & 31)
    addr = AIC_BASE + AIC_MASK_SET + reg_off
    cur = p.read32(addr)
    p.write32(addr, bit)  # AIC MASK_SET: writing 1 sets that bit
    after = p.read32(addr)
    # HW_STATE at same offset in its block:
    hw_addr = AIC_BASE + AIC_HW_STATE + reg_off
    hw = p.read32(hw_addr)
    log(f"  AIC mask irq={irq} reg+{reg_off:#x} bit={bit:#x} "
        f"(pre={cur:#x} post={after:#x} hw={hw:#x})")


def aic_drain_events(p, max_pulls=8):
    log(f"  draining AIC EVENT @ {AIC_BASE + AIC_EVENT:#x}")
    for i in range(max_pulls):
        ev = p.read32(AIC_BASE + AIC_EVENT)
        if ev == 0:
            log(f"    event[{i}] = 0 (drained)")
            break
        die = (ev >> 24) & 0xff
        typ = (ev >> 16) & 0xff
        num = ev & 0xffff
        log(f"    event[{i}] = {ev:#x}  (die={die} type={typ} num={num})")


def snap_asc(p, base, tag):
    cc = p.read32(base + 0x44)
    cs = p.read32(base + 0x48)
    ib = p.read32(base + 0x8110)
    ob = p.read32(base + 0x8114)
    b14 = p.read32(base + 0xb14)
    log(f"[{tag}] CC={cc:#x} CS={cs:#x} IB={ib:#x} OB={ob:#x} +b14={b14:#x}")
    return dict(cc=cc, cs=cs, ib=ib, ob=ob, b14=b14)


def main():
    var = int(os.environ.get("BATOS_AOP_VAR", "1"))
    log(f"variant={var}  (0=baseline 1=mask-aop 2=mask+drain 3=+dart)")

    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    if os.environ.get("BATOS_SKIP_BOOTSTRAP", "0") != "1":
        log("chainloading patched m1n1...")
        chainload(iface, p, u)
        u = ProxyUtils(p, heap_size=128 * 1024 * 1024)
        log("  patched m1n1 up")

    # WDT disable
    log("disabling M4 AP watchdog...")
    for addr, val in [(0x3882BC224, 0), (0x3882B8008, 0xffffffff),
                      (0x3882B802C, 0xffffffff), (0x3882B8020, 0xffffffff)]:
        try:
            p.write32(addr, val)
        except Exception as e:
            log(f"  wdt err {addr:#x}: {e!r}")

    # PMGR power enable (expected to error on M4 — tolerate)
    for path in ("/arm-io/aop", "/arm-io/dart-aop"):
        try:
            rc = p.pmgr_adt_power_enable(path)
            log(f"  pmgr {path}: rc={rc}")
        except Exception as e:
            log(f"  pmgr {path}: {type(e).__name__}: {e}")

    aop_node = u.adt["/arm-io/aop"]
    aop_base = aop_node.get_reg(0)[0]
    aop_irqs = list(aop_node.interrupts)
    log(f"AOP @ {aop_base:#x}  IRQs={aop_irqs}")

    # Stage firmware (__DATA + __OS_LOG; __TEXT + __ETEXT iBoot-staged)
    sr = getattr(aop_node, "segment-ranges", None)
    names_raw = getattr(aop_node, "segment-names", b"")
    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_segs = parse_adt_segments(sr)
    adt_by = dict(zip(names, adt_segs))

    macho = AOP_BLOB.read_bytes()
    mc_by = {seg[0]: seg for seg in macho_segs_fn(macho)}

    skip = set()
    for nm in ("__TEXT", "__ETEXT"):
        if nm not in mc_by or nm not in adt_by:
            continue
        m = mc_by[nm]; a = adt_by[nm]
        ok = True
        for off in (0x0, 0x100, 0x200):
            if off >= m[4]: break
            exp = macho[m[3]+off:m[3]+off+16]
            got = iface.readmem(a["phys"]+off, 16)
            if got != exp:
                ok = False
                break
        if ok:
            skip.add(nm)
            log(f"  {nm}: iBoot-staged (skip)")

    for nm in names:
        if nm in skip: continue
        if nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0: continue
        if m[4] > a["size"]: log(f"  {nm}: OVERFLOW"); continue
        t = time.time()
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], macho[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], macho[m[3]:m[3]+m[4]])
        log(f"  {nm}: {m[4]}B -> {a['phys']:#x}  ({(time.time()-t)*1000:.0f}ms)")

    # DART
    from m1n1.hw.dart import DART
    dart_node = u.adt["/arm-io/dart-aop"]
    vm_base = getattr(dart_node, "vm-base", None) or 0x8000
    log(f"DART vm_base={vm_base:#x}")
    dart = DART.from_adt(u, "/arm-io/dart-aop",
                         iova_range=(vm_base, 0x1000000000))
    dart.initialize()

    from m1n1.fw.aop.client import AOPClient
    aop = AOPClient(u, "/arm-io/aop", dart)
    aop.verbose = 3

    # Reset RUN=0 if already running
    pre_cc = p.read32(aop_base + 0x44)
    if pre_cc & 0x10:
        log(f"AOP was running (CC={pre_cc:#x}) — clearing RUN")
        p.write32(aop_base + 0x44, pre_cc & ~0x10)
        time.sleep(0.1)

    # Snap before bootargs
    snap_asc(p, aop_base, "pre-args")

    # Bootargs
    try:
        args = aop.read_bootargs()
        log(f"bootargs keys ({len(list(args.keys()))}): {list(args.keys())}")
    except Exception as e:
        log(f"read_bootargs fail: {e!r}")
        os._exit(1)

    log("update_bootargs({p0CE,laCn,tPOA,gila})...")
    try:
        aop.update_bootargs({'p0CE':0x20000,'laCn':0x0,'tPOA':0x1,'gila':0x80})
    except Exception as e:
        log(f"update_bootargs err: {e!r}")

    # Reset OUTBOX_CTRL per reference
    p.write32(aop_base + 0x8114, 0x20001)
    log(f"OUTBOX_CTRL reset: {p.read32(aop_base + 0x8114):#x}")

    # === AIC PHASE ===
    if var >= 1:
        log("AIC: snapshotting HW_STATE for AOP IRQs before mask...")
        for irq in aop_irqs:
            ro = (irq >> 5) * 4
            hw = p.read32(AIC_BASE + AIC_HW_STATE + ro)
            ms = p.read32(AIC_BASE + AIC_MASK_SET + ro)
            log(f"  irq={irq} reg+{ro:#x} HW_STATE={hw:#x} MASK_SET(pre)={ms:#x}")

        log("AIC: masking AOP IRQs...")
        for irq in aop_irqs:
            aic_mask_irq(p, irq)

    if var >= 3:
        log("AIC: masking dart-aop IRQ too...")
        dart_irqs = list(u.adt["/arm-io/dart-aop"].interrupts)
        for irq in dart_irqs:
            aic_mask_irq(p, irq)

    if var >= 2:
        aic_drain_events(p)

    snap_asc(p, aop_base, "pre-start")

    log("aop.start()...")
    t0 = time.time()
    try:
        aop.start()
        log(f"*** AOP START OK in {(time.time()-t0)*1000:.0f}ms ***")
        for epno in [0x20, 0x21, 0x22, 0x24, 0x25, 0x26, 0x27, 0x28]:
            try:
                aop.start_ep(epno)
                log(f"  ep {epno:#x} started")
            except Exception as e:
                log(f"  ep {epno:#x}: {type(e).__name__}: {e}")
    except Exception as e:
        log(f"aop.start() FAIL: {type(e).__name__}: {e}")
        snap_asc(p, aop_base, "post-fail")
        log(f"  iop_power={aop.mgmt.iop_power_state:#x} "
            f"ap_power={aop.mgmt.ap_power_state:#x}")
        if var >= 1:
            log("AIC post-fail state:")
            for irq in aop_irqs:
                ro = (irq >> 5) * 4
                hw = p.read32(AIC_BASE + AIC_HW_STATE + ro)
                ms = p.read32(AIC_BASE + AIC_MASK_SET + ro)
                log(f"  irq={irq} HW_STATE={hw:#x} MASK_SET={ms:#x}")
            aic_drain_events(p)

    os._exit(0)


if __name__ == "__main__":
    main()
