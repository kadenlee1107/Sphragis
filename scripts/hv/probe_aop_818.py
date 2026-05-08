#!/usr/bin/env python3
"""Study AOP +0x818 counter pattern — FW is incrementing this on
every interaction. Does the value reflect msg contents?

Also check: does INBOX msg actually affect anything, or is +0x818
just counting every MMIO touch?
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
from m1n1 import asm

AOP_BLOB    = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
M1N1_MACHO  = ROOT / "external/m1n1/build/m1n1.macho"
AOP = 0x390600000


def log(m): print(f"[818] {m}", flush=True)


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


def main():
    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    log("chainloading...")
    chainload(iface, p, u)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    try: p.dapf_init("/arm-io/dart-aop")
    except: pass

    # Stage
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
            if iface.readmem(a["phys"]+off, 16) != macho[m[3]+off:m[3]+off+16]:
                ok = False; break
        if ok: skip.add(nm)
    for nm in names:
        if nm in skip or nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0 or m[4] > a["size"]: continue
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], macho[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], macho[m[3]:m[3]+m[4]])

    try:
        from m1n1.fw.aop.base import AOPBase
        AOPBase(u).update_bootargs({'p0CE': 0x20000, 'laCn': 0,
                                    'tPOA': 1, 'gila': 0x80})
    except: pass

    p.write32(AOP + 0x44, 0x10)
    time.sleep(0.5)  # let FW init
    baseline = p.read32(AOP + 0x818)
    log(f"baseline +0x818 after 500ms idle: {baseline:#x}")

    # Study 1: Pure reads — do reads increment +0x818?
    log("\nS1: 10 reads of CPU_STATUS — does +0x818 increment?")
    before = p.read32(AOP + 0x818)
    for _ in range(10):
        _ = p.read32(AOP + 0x48)
    after = p.read32(AOP + 0x818)
    log(f"  before={before:#x} after 10 reads={after:#x} delta={after-before:#x}")

    # Study 2: pure INBOX write — how much does +0x818 change?
    log("\nS2: single INBOX write")
    before = p.read32(AOP + 0x818)
    p.write64(AOP + 0x8800, (6 << 52) | 0x220)
    p.write64(AOP + 0x8808, 0)
    time.sleep(0.3)
    after = p.read32(AOP + 0x818)
    log(f"  before={before:#x} after INBOX={after:#x} delta={after-before:#x}")

    # Study 3: INBOX write with DIFFERENT payload — does counter reflect msg?
    log("\nS3: vary INBOX payload, check +0x818")
    for payload in [0x0, 0xdeadbeef, 0xffffffff, (1 << 52), (3 << 52)]:
        before = p.read32(AOP + 0x818)
        ob_before = p.read32(AOP + 0x8114)
        p.write64(AOP + 0x8800, payload)
        p.write64(AOP + 0x8808, 0)
        time.sleep(0.2)
        after = p.read32(AOP + 0x818)
        ob_after = p.read32(AOP + 0x8114)
        log(f"  payload={payload:#x}: +0x818 {before:#x}→{after:#x} (d={after-before:#x}) "
            f"OB {ob_before:#x}→{ob_after:#x}")
        if not (ob_after & (1 << 17)):
            m0 = p.read64(AOP + 0x8830)
            m1 = p.read64(AOP + 0x8838)
            log(f"    *** OUTBOX msg! m0={m0:#x} m1={m1:#x} ***")
            os._exit(0)

    # Study 4: Is the counter COUNTING our reads? Read 0x818 many times.
    log("\nS4: consecutive 0x818 reads")
    vals = [p.read32(AOP + 0x818) for _ in range(5)]
    log(f"  5 reads: {[hex(v) for v in vals]}")

    # Study 5: Dump reg[4] post-init to see if FW wrote code there
    log("\nS5: dump reg[4] post-init, compare to pre-init")
    reg4 = iface.readmem(0x390c62000, 256)
    log(f"  reg[4] first 128 bytes: {reg4[:128].hex()}")

    # Study 6: scan ENTIRE AOP reg[0] 0x0..0x2000 for non-zero regs
    log("\nS6: full AOP reg[0]+0x0..0x2000 scan")
    interesting = []
    for off in range(0, 0x2000, 4):
        try:
            v = p.read32(AOP + off)
            if v != 0 and v != 0xffffffff:
                interesting.append((off, v))
        except: break
    log(f"  {len(interesting)} non-trivial values")
    for off, v in interesting:
        log(f"    +{off:#x} = {v:#x}")

    # Study 7: read __DATA+0x498 again — did the table grow/change?
    log("\nS7: __DATA+0x498 final state")
    data_phys = adt_by['__DATA']['phys']
    final = iface.readmem(data_phys + 0x498, 512)
    nz_pointers = [struct.unpack('<Q', final[i:i+8])[0] for i in range(0, 512, 8)]
    nz = [x for x in nz_pointers if x != 0]
    log(f"  {len(nz)} non-zero u64 entries")
    for i, v in enumerate(nz_pointers[:16]):
        if v:
            log(f"    [{i}] = {v:#x}")

    os._exit(0)


if __name__ == "__main__":
    main()
