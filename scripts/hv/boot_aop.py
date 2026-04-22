#!/usr/bin/env python3
"""Stage + boot AOP ASC, as prereq for MTP boot.

Theory: MTP hangs after reading one INBOX msg because it needs
AOP running (mtp-aop-mux ADT compatible 'hid-transport,mux'). This
script boots AOP first. If AOP Hellos cleanly, next step is MTP.

AOP firmware at firmware/aop/aopfw-mac16gaop.RELEASE.bin is a raw
Mach-O (no rkosftab wrapper like MTP has). 6 populated segments:
  __TEXT   vm=0x01000000 fs=0xcd000  (87% non-zero)
  __DATA   vm=0x010cd000 fs=0xf9000  (7.7%)
  __ETEXT  vm=0x011c6000 fs=0x16000  (89%)
  __OS_LOG vm=0xfd000000 fs=0x2a000  (98%)
  __MISC   vm=0xfe000000 fs=0x1000   (0.3%)
  __CMA    vm=0xff000000 fs=0x2000   (53%)

Sequence (chainload patched m1n1 → WDT off → stage → boot):
  1. probe /arm-io/aop ADT for segment-ranges
  2. verify __TEXT iBoot-staged (same XOM pattern as MTP)
  3. stage remaining segments via iface.writemem
  4. SMC + DART-AOP setup
  5. StandardASC + mgmt.start → wait_boot
"""
import os, pathlib, struct, sys, time

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


def macho_segs(macho_bytes):
    ncmds = struct.unpack("<I", macho_bytes[16:20])[0]
    cur = 32
    segs = []
    for _ in range(ncmds):
        cmd, sz = struct.unpack("<II", macho_bytes[cur:cur+8])
        if cmd == 0x19:
            name = macho_bytes[cur+8:cur+24].rstrip(b"\x00").decode()
            vm, vmsz, fo, fs = struct.unpack("<QQQQ", macho_bytes[cur+24:cur+56])
            segs.append((name, vm, vmsz, fo, fs))
        cur += sz
    return segs


def parse_adt_segments(raw):
    segs = []
    for i in range(len(raw) // 32):
        s = raw[i*32:(i+1)*32]
        phys, iova, remap, size = struct.unpack("<QQQI4x", s)
        segs.append({"phys": phys, "iova": iova, "size": size})
    return segs


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
    try:
        p.write32(0x3882BC224, 0)
        p.write32(0x3882B8008, 0xffffffff)
        p.write32(0x3882B802C, 0xffffffff)
        p.write32(0x3882B8020, 0xffffffff)
    except Exception as e:
        log(f"  wdt err: {e!r}")

    # Probe AOP ADT
    aop_node = u.adt["/arm-io/aop"]
    aop_base = aop_node.get_reg(0)[0]
    log(f"AOP @ {aop_base:#x}  compat={list(getattr(aop_node, 'compatible', []))}")

    sr = getattr(aop_node, "segment-ranges", None)
    if sr is None:
        log("ABORT: /arm-io/aop has no segment-ranges property")
        # Try nub instead
        try:
            nub = u.adt["/arm-io/aop/iop-aop-nub"]
            sr = getattr(nub, "segment-ranges", None)
            names_raw = getattr(nub, "segment-names", b"")
            log(f"  using nub's segment-ranges ({len(sr)}B)")
        except Exception as e:
            log(f"  no nub either: {e!r}")
            os._exit(1)
    else:
        names_raw = getattr(aop_node, "segment-names", b"")

    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_segs = parse_adt_segments(sr)
    log(f"ADT segments ({len(adt_segs)}):")
    adt_by = {}
    for nm, s in zip(names, adt_segs):
        log(f"  {nm:>12s}  phys={s['phys']:#014x}  iova={s['iova']:#014x}  size={s['size']:#x}")
        adt_by[nm] = s

    # Parse Mach-O
    if not AOP_BLOB.exists():
        log(f"ABORT: {AOP_BLOB} missing")
        os._exit(1)
    macho = AOP_BLOB.read_bytes()
    mc_by = {}
    for seg in macho_segs(macho):
        mc_by[seg[0]] = seg
        log(f"  mach-o {seg[0]:>12s} vm={seg[1]:#x} fs={seg[4]:#x}")

    # CPU state
    cc = p.read32(aop_base + 0x44)
    cs = p.read32(aop_base + 0x48)
    log(f"AOP pre-stage: CPU_CONTROL={cc:#x} CPU_STATUS={cs:#x}")

    # Verify __TEXT and __ETEXT (both are code — iBoot may have staged
    # both as write-protected. If __ETEXT also matches Mach-O, skip
    # staging it to avoid SYNC exceptions).
    skip_segs = set()
    for seg_name in ("__TEXT", "__ETEXT"):
        if seg_name not in mc_by or seg_name not in adt_by:
            continue
        t_mc = mc_by[seg_name]
        t_adt = adt_by[seg_name]
        all_match = True
        for off in (0x0, 0x100, 0x200):
            if off >= t_mc[4]:
                break
            exp = macho[t_mc[3]+off:t_mc[3]+off+16]
            got = iface.readmem(t_adt["phys"]+off, 16)
            match = "MATCH" if got == exp else "MISMATCH"
            log(f"  {seg_name}[+{off:#x}] iBoot={got.hex()} macho={exp.hex()} [{match}]")
            if got != exp:
                all_match = False
        if all_match:
            skip_segs.add(seg_name)
            log(f"  {seg_name}: iBoot-staged, skipping host write")
        elif seg_name == "__TEXT" and os.environ.get("BATOS_AOP_FORCE_TEXT", "0") != "1":
            log("ABORT: __TEXT mismatch (set BATOS_AOP_FORCE_TEXT=1 to override)")
            os._exit(1)

    # Stage all non-__TEXT segments present in both. Use
    # compressed_writemem for big segments (~10x faster on AOP's
    # 996KB __DATA + 168KB __OS_LOG).
    staged = []
    for nm in names:
        if nm in skip_segs:
            continue
        if nm not in mc_by or nm not in adt_by:
            log(f"  {nm}: no match in both — skip")
            continue
        mc = mc_by[nm]
        ad = adt_by[nm]
        fo, fs = mc[3], mc[4]
        if fs == 0:
            log(f"  {nm}: filesize=0 — skip")
            continue
        if fs > ad["size"]:
            log(f"  {nm}: OVERFLOW ({fs} > {ad['size']}) — skip")
            continue
        payload = macho[fo:fo+fs]
        # compressed_writemem uses gzdec on m1n1 side — big segments
        # ship 10x faster since zero-heavy content compresses well.
        t = time.time()
        if fs >= 64 * 1024:
            u.compressed_writemem(ad["phys"], payload, True)
        else:
            iface.writemem(ad["phys"], payload)
        log(f"  {nm}: {fs}B -> {ad['phys']:#x} OK ({(time.time()-t)*1000:.0f}ms)")
        staged.append(nm)

    # SMC + DART-AOP
    from m1n1.fw.smc import SMCClient
    from m1n1.hw.dart import DART
    try:
        smc_addr = u.adt["arm-io/smc"].get_reg(0)[0]
        smc = SMCClient(u, smc_addr)
        smc.start(); smc.start_ep(0x20); smc.verbose = 0
        log(f"SMC up @ {smc_addr:#x}")
    except Exception as e:
        log(f"SMC err (continuing): {e!r}")
        smc = None

    try:
        dart = DART.from_adt(u, "/arm-io/dart-aop", iova_range=(0x8000, 0x100000))
        dart.dart.regs.TCR[1].set(BYPASS_DAPF=1, BYPASS_DART=0, TRANSLATE_ENABLE=1)
        try:
            dart.initialize()
        except Exception:
            pass
        log("DART-AOP set up")
    except Exception as e:
        log(f"DART-AOP err: {e!r}")
        dart = None

    # StandardASC boot
    from m1n1.fw.asc import StandardASC
    from m1n1.fw.asc.base import ASCTimeout
    aop = StandardASC(u, aop_base, dart, stream=1)
    aop.verbose = 2
    aop.allow_phys = True

    log("kicking AOP: CPU_CONTROL.RUN=1 + mgmt.start(SetIOPPower=0x220)...")
    try:
        aop.asc.CPU_CONTROL.set(RUN=1)
        aop.mgmt.start()
    except Exception as e:
        log(f"kick err: {e!r}")

    # Watch for Hello / power-state progress
    deadline = time.time() + 20
    last_snap = time.time()
    t0 = time.time()
    while time.time() < deadline:
        if (aop.mgmt.iop_power_state == 0x20 and
                aop.mgmt.ap_power_state == 0x20):
            log(f"AOP BOOT OK in {(time.time()-t0)*1000:.0f}ms!")
            break
        try:
            aop.work()
        except Exception as e:
            log(f"work err: {e!r}")
            break
        if time.time() - last_snap > 1.0:
            cc_s = p.read32(aop_base + 0x44)
            cs_s = p.read32(aop_base + 0x48)
            ob_s = p.read32(aop_base + 0x8114)
            ic_s = p.read32(aop_base + 0x8110)
            b14 = p.read32(aop_base + 0x0b14)
            log(f"  t={int((time.time()-t0)*1000):4d}ms CC={cc_s:#x} CS={cs_s:#x} "
                f"IB={ic_s:#x} OB={ob_s:#x} +b14={b14:#x} "
                f"iop={aop.mgmt.iop_power_state:#x} ap={aop.mgmt.ap_power_state:#x}")
            last_snap = time.time()
    else:
        log(f"AOP BOOT TIMEOUT after 20s")
        log(f"  final CS={p.read32(aop_base+0x48):#x} +b14={p.read32(aop_base+0xb14):#x}")

    # Diagnostic dump regardless
    log("post-boot IMPL reg non-zero in 0x100..0x800:")
    for off in range(0x100, 0x800, 4):
        v = p.read32(aop_base + off)
        if v:
            log(f"    [+{off:#x}] = {v:#x}")

    os._exit(0)


if __name__ == "__main__":
    main()
