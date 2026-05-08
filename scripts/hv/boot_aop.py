#!/usr/bin/env python3
"""Stage + boot AOP ASC using m1n1's AOPClient.

Previous attempt used StandardASC and hung at FW-waits-for-something.
Looking at reference experiments/aop_als.py reveals AOP needs:
  1. pmgr_adt_power_enable for /arm-io/aop + /arm-io/dart-aop
  2. DART with specific iova range from adt vm-base
  3. dart.initialize()
  4. AOPClient (subclass of StandardASC) + update_bootargs
     — writes config keys (p0CE, laCn, tPOA, gila) into a DRAM
     bootargs region that the FW reads during early init
  5. aop.start() → mgmt.start + wait_boot (3s timeout default)

The bootargs blob is what our previous StandardASC attempt missed.
FW wouldn't even consume our INBOX msg because it hadn't initialized
past reading bootargs.
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


def log(m):
    print(f"[aop] {m}", flush=True)


def main():
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

    # PMGR power enable (reference aop_als.py does this — NEW vs MTP)
    log("pmgr_adt_power_enable for aop + dart-aop...")
    for path in ("/arm-io/aop", "/arm-io/dart-aop"):
        try:
            rc = p.pmgr_adt_power_enable(path)
            log(f"  {path}: rc={rc}")
        except Exception as e:
            log(f"  {path}: {type(e).__name__}: {e}")

    # Stage firmware into __DATA / __OS_LOG (skip __TEXT + __ETEXT — iBoot)
    aop_node = u.adt["/arm-io/aop"]
    aop_base = aop_node.get_reg(0)[0]
    log(f"AOP @ {aop_base:#x}")

    sr = getattr(aop_node, "segment-ranges", None)
    if sr is None:
        log("ABORT: /arm-io/aop lacks segment-ranges")
        os._exit(1)
    names_raw = getattr(aop_node, "segment-names", b"")
    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_segs = parse_adt_segments(sr)
    adt_by = dict(zip(names, adt_segs))
    log(f"  ADT segs: {list(adt_by.keys())}")
    for nm, s in adt_by.items():
        log(f"    {nm:>10s}  phys={s['phys']:#014x}  iova={s['iova']:#014x}  size={s['size']:#x}")

    macho = AOP_BLOB.read_bytes()
    mc_by = {seg[0]: seg for seg in macho_segs_fn(macho)}

    # Verify __TEXT + __ETEXT are iBoot-staged; skip write for both.
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
                log(f"  {nm}[+{off:#x}] MISMATCH")
                break
        if ok:
            skip.add(nm)
            log(f"  {nm}: iBoot-staged (skip)")

    # Stage everything else
    for nm in names:
        if nm in skip:
            continue
        if nm not in mc_by or nm not in adt_by:
            continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0:
            continue
        if m[4] > a["size"]:
            log(f"  {nm}: OVERFLOW"); continue
        t = time.time()
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], macho[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], macho[m[3]:m[3]+m[4]])
        log(f"  {nm}: {m[4]}B -> {a['phys']:#x}  ({(time.time()-t)*1000:.0f}ms)")

    # DART with vm-base from ADT
    from m1n1.hw.dart import DART
    dart_node = u.adt["/arm-io/dart-aop"]
    vm_base = getattr(dart_node, "vm-base", None)
    if vm_base is None:
        vm_base = 0x8000
    log(f"DART vm_base={vm_base:#x}")
    dart = DART.from_adt(u, "/arm-io/dart-aop",
                         iova_range=(vm_base, 0x1000000000))
    dart.initialize()

    # Create AOPClient
    from m1n1.fw.aop.client import AOPClient
    aop = AOPClient(u, "/arm-io/aop", dart)
    aop.verbose = 3

    # Reset FW if running so bootargs changes get re-read on next boot.
    # If RUN=1 already (from a previous invocation), the FW has already
    # consumed bootargs; writing new values has no effect until we flip
    # RUN=0 → update bootargs → RUN=1.
    pre_cc = p.read32(aop_base + 0x44)
    if pre_cc & 0x10:
        log(f"AOP was running (CC={pre_cc:#x}) — clearing RUN for re-read")
        p.write32(aop_base + 0x44, pre_cc & ~0x10)
        time.sleep(0.1)
        log(f"  CC after clear: {p.read32(aop_base + 0x44):#x}  "
            f"CS: {p.read32(aop_base + 0x48):#x}")

    # Read + dump current bootargs
    try:
        args = aop.read_bootargs()
        log("bootargs present — keys:")
        for k in list(args.keys())[:20]:
            log(f"    {k!r}")
    except Exception as e:
        log(f"read_bootargs failed: {type(e).__name__}: {e}")
        log("FW may need initial CPU_CONTROL.RUN=1 before bootargs region is valid")
        # Try anyway — some iBoot handoffs pre-populate
        try:
            aop.asc.CPU_CONTROL.set(RUN=1)
            time.sleep(0.1)
            args = aop.read_bootargs()
            log(f"after RUN=1, bootargs keys: {list(args.keys())[:20]}")
        except Exception as e2:
            log(f"still failing: {e2!r}")
            os._exit(1)

    # Update bootargs with reference values
    log("update_bootargs({p0CE: 0x20000, laCn: 0, tPOA: 1, gila: 0x80})...")
    try:
        aop.update_bootargs({
            'p0CE': 0x20000,
            'laCn': 0x0,
            'tPOA': 0x1,
            'gila': 0x80,
        })
    except Exception as e:
        log(f"update_bootargs err: {type(e).__name__}: {e}")

    # Skip dapf (hangs on M4 dart-mtp but may be ok for dart-aop — try?)
    skip_dapf = os.environ.get("BATOS_SKIP_DAPF", "1") == "1"
    if not skip_dapf:
        log("dapf_init_all (30s timeout)...")
        saved = iface.dev.timeout
        iface.dev.timeout = 30
        try:
            p.dapf_init_all()
            log("  dapf_init_all OK")
        except Exception as e:
            log(f"  dapf_init_all fail: {type(e).__name__}: {e}")
        iface.dev.timeout = saved

    # Reset OUTBOX_CTRL per reference
    try:
        p.write32(aop_base + 0x8114, 0x20001)
        log(f"OUTBOX_CTRL reset: {p.read32(aop_base + 0x8114):#x}")
    except Exception as e:
        log(f"OB reset err: {e!r}")

    # aop.start() = StandardASC.start() = boot + mgmt.start + wait_boot(3)
    log("aop.start()...")
    t0 = time.time()
    try:
        aop.start()
        log(f"AOP START OK in {(time.time()-t0)*1000:.0f}ms")
        # start endpoints
        for epno in [0x20, 0x21, 0x22, 0x24, 0x25, 0x26, 0x27, 0x28]:
            try:
                aop.start_ep(epno)
                log(f"  ep {epno:#x} started")
            except Exception as e:
                log(f"  ep {epno:#x}: {type(e).__name__}")
    except Exception as e:
        log(f"aop.start() FAILED: {type(e).__name__}: {e}")
        cc = p.read32(aop_base + 0x44)
        cs = p.read32(aop_base + 0x48)
        ic = p.read32(aop_base + 0x8110)
        ob = p.read32(aop_base + 0x8114)
        b14 = p.read32(aop_base + 0xb14)
        log(f"  CC={cc:#x} CS={cs:#x} IB={ic:#x} OB={ob:#x} +b14={b14:#x}")
        log(f"  iop_power={aop.mgmt.iop_power_state:#x} ap_power={aop.mgmt.ap_power_state:#x}")

    os._exit(0)


if __name__ == "__main__":
    main()
