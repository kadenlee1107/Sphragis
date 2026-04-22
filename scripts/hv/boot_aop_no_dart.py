#!/usr/bin/env python3
"""Boot AOP WITHOUT calling dart.initialize() — let iBoot's config stand.

Hypothesis: DART.initialize() in our previous scripts wipes TCR[0..14]
to blank TRANSLATE_ENABLE=1 and invalidates all TTBRs. iBoot configured
DART-AOP with specific streams for the FW's DMA targets; when we clobber
it, FW traps trying to access OS_LOG / telemetry DMA.

Evidence: post-boot_aop.py scan showed DART-AOP reg[0]+0x0..+0x200 all
zero, while DART-MTP (untouched this session) still shows dense iBoot
config at +0x0..+0xb0.

This script:
  1. Chainload patched m1n1
  2. Disable AP watchdog
  3. Stage __DATA + __OS_LOG (same as before — __TEXT/__ETEXT iBoot-staged)
  4. Update bootargs (same 4 reference keys)
  5. Reset OUTBOX_CTRL
  6. Kick RUN=1 DIRECTLY (no dart.initialize, no AOPClient overhead)
  7. Poll for OUTBOX Hello from FW, process it manually

Run only on fresh power-cycle — previous attempts leave AOP in stuck state.
"""
import os, pathlib, struct, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

AOP_BLOB = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
M1N1_MACHO = ROOT / "external/m1n1/build/m1n1.macho"


def log(m): print(f"[no-dart] {m}", flush=True)


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


AOP = 0x390600000

def snap(p, tag):
    cc = p.read32(AOP + 0x44)
    cs = p.read32(AOP + 0x48)
    ib = p.read32(AOP + 0x8110)
    ob = p.read32(AOP + 0x8114)
    b14 = p.read32(AOP + 0xb14)
    u40 = p.read32(AOP + 0x40)
    r818 = p.read32(AOP + 0x818)
    log(f"[{tag}] CC={cc:#x} CS={cs:#x} IB={ib:#x} OB={ob:#x} b14={b14:#x} "
        f"+0x40={u40:#x} +0x818={r818:#x}")


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

    for addr, val in [(0x3882BC224, 0), (0x3882B8008, 0xffffffff),
                      (0x3882B802C, 0xffffffff), (0x3882B8020, 0xffffffff)]:
        try: p.write32(addr, val)
        except: pass

    snap(p, "pre-stage")

    # Stage segments
    aop_node = u.adt["/arm-io/aop"]
    sr = getattr(aop_node, "segment-ranges", None)
    names_raw = getattr(aop_node, "segment-names", b"")
    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_by = dict(zip(names, parse_adt_segments(sr)))
    macho = AOP_BLOB.read_bytes()
    mc_by = {seg[0]: seg for seg in macho_segs_fn(macho)}

    skip = set()
    for nm in ("__TEXT", "__ETEXT"):
        if nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        ok = True
        for off in (0x0, 0x100, 0x200):
            if off >= m[4]: break
            exp = macho[m[3]+off:m[3]+off+16]
            got = iface.readmem(a["phys"]+off, 16)
            if got != exp: ok = False; break
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
        log(f"  {nm}: {m[4]}B -> {a['phys']:#x} ({(time.time()-t)*1000:.0f}ms)")

    # Bootargs (no DART init needed for this — bootargs in DRAM)
    from m1n1.fw.aop.base import AOPBase
    aopb = AOPBase(u)
    try:
        args = aopb.read_bootargs()
        log(f"bootargs: {len(list(args.keys()))} keys")
        aopb.update_bootargs({'p0CE':0x20000,'laCn':0,'tPOA':1,'gila':0x80})
        log("bootargs updated")
    except Exception as e:
        log(f"bootargs err: {e!r}")
        os._exit(1)

    # Reset OUTBOX_CTRL (per aop_als.py reference)
    p.write32(AOP + 0x8114, 0x20001)

    # CRITICAL: DO NOT call dart.initialize() — leave iBoot's config alone
    log("*** SKIPPING dart.initialize() — leaving iBoot DART setup intact ***")

    snap(p, "pre-RUN")

    # Kick RUN=1
    log("CC.RUN=1...")
    p.write32(AOP + 0x44, 0x10)

    # Poll for Hello from FW on OUTBOX
    log("polling OUTBOX for Hello (5s timeout)...")
    deadline = time.time() + 5
    ob_ctrl = None
    found_msg = False
    while time.time() < deadline:
        ob = p.read32(AOP + 0x8114)
        if ob_ctrl != ob:
            ob_ctrl = ob
            log(f"  OB_CTRL={ob:#x}")
        if not (ob & (1 << 17)):  # not EMPTY
            ob0 = p.read32(AOP + 0x8830) | (p.read32(AOP + 0x8834) << 32)
            ob1 = p.read32(AOP + 0x8838) | (p.read32(AOP + 0x883c) << 32)
            log(f"  *** OUTBOX Hello: msg0={ob0:#x} msg1={ob1:#x} ***")
            found_msg = True
            break
        time.sleep(0.05)

    snap(p, "final")

    if not found_msg:
        log("NO HELLO from FW. +0x40 stage stuck?")
        u40 = p.read32(AOP + 0x40)
        log(f"  +0x40 (boot stage) = {u40:#x} "
            f"{'(advanced!)' if u40 != 0xa0000 else '(stuck at pre-boot)'}")

    os._exit(0)


if __name__ == "__main__":
    main()
