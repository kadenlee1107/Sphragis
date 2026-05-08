#!/usr/bin/env python3
"""Chase Mgmt_Ping (TYPE=3) response. 818 probe showed TYPE=3 reset
the +0x818 counter to 0 — FW handled it specially. Maybe FW writes
its Pong response somewhere other than OUTBOX — another MMIO reg,
or in DRAM via DART.

Strategy: hash all major FW-writable regions BEFORE sending Ping,
then AFTER, and diff. Any changed bytes ARE FW's response footprint.
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


def log(m): print(f"[p] {m}", flush=True)


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


def chunks(iface, regions, chunk_size=4096):
    """Hash + return bytes for each region."""
    result = {}
    for name, base, size in regions:
        try:
            # Cap to 64 KB per region to keep it fast
            read_size = min(size, 65536)
            data = iface.readmem(base, read_size)
            result[name] = (base, data)
        except Exception as e:
            log(f"  read {name}: {type(e).__name__}")
    return result


def diff(a, b, name):
    """Find byte-level diffs between two readings."""
    ba, da = a
    bb, db = b
    if ba != bb or len(da) != len(db):
        log(f"  {name}: read geometry differs")
        return
    changed = []
    for i, (x, y) in enumerate(zip(da, db)):
        if x != y:
            changed.append((i, x, y))
    log(f"  {name} @{ba:#x}: {len(changed)} byte diffs")
    # Group adjacent diffs
    if changed:
        groups = [[changed[0]]]
        for c in changed[1:]:
            if c[0] == groups[-1][-1][0] + 1:
                groups[-1].append(c)
            else:
                groups.append([c])
        for g in groups[:8]:
            start = g[0][0]
            length = len(g)
            log(f"    +{start:#x}..+{start+length:#x} "
                f"before={bytes(x[1] for x in g).hex()} "
                f"after={bytes(x[2] for x in g).hex()}")


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

    # Start FW + let it stabilize
    p.write32(AOP + 0x44, 0x10)
    time.sleep(1.0)

    # BEFORE Ping — capture state
    log("\n=== BEFORE ping ===")
    data_phys = adt_by['__DATA']['phys']
    os_log_phys = adt_by['__OS_LOG']['phys']
    # Small region sizes only — wide reads SError on undefined MMIO
    regions = [
        ("__DATA+0x400..0x600",     data_phys + 0x400, 0x200),
        ("__DATA+0x600..0x800",     data_phys + 0x600, 0x200),
        ("__OS_LOG+0..0x400",       os_log_phys, 0x400),
    ]
    before = chunks(iface, regions)
    b818 = p.read32(AOP + 0x818)
    bob = p.read32(AOP + 0x8114)
    log(f"  +0x818 = {b818:#x}  OB_CTRL = {bob:#x}")

    # SEND Mgmt_Ping
    log("\n=== Sending Mgmt_Ping (TYPE=3) ===")
    p.write64(AOP + 0x8800, (3 << 52))
    p.write64(AOP + 0x8808, 0)
    time.sleep(0.5)

    # AFTER
    a818 = p.read32(AOP + 0x818)
    aob = p.read32(AOP + 0x8114)
    log("\n=== AFTER ping ===")
    log(f"  +0x818 = {a818:#x} (delta {a818-b818:+#x})  OB_CTRL = {aob:#x}")

    log("\ndiffing regions:")
    after = chunks(iface, regions)
    for key in before:
        if key in after:
            diff(before[key], after[key], key)

    # Also check OUTBOX regs directly
    log(f"\n  OB_CTRL={p.read32(AOP + 0x8114):#x}")
    log(f"  OUTBOX0_lo={p.read32(AOP + 0x8830):#x}")
    log(f"  OUTBOX0_hi={p.read32(AOP + 0x8834):#x}")
    log(f"  OUTBOX1_lo={p.read32(AOP + 0x8838):#x}")
    log(f"  OUTBOX1_hi={p.read32(AOP + 0x883c):#x}")

    # Also dump CPU_STATUS history (maybe state changes)
    log(f"\n  CPU_STATUS now: {p.read32(AOP + 0x48):#x}")

    os._exit(0)


if __name__ == "__main__":
    main()
